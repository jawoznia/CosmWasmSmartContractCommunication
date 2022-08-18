#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use cosmwasm_std::{coins, from_binary, Addr, Decimal, Empty, StdError};
    use cw_multi_test::{next_block, App, AppResponse, Contract, ContractWrapper, Executor};
    use msgs::admin::{
        AdminsListResp, ExecuteMsg as AdminExecuteMsg, InstantiateMsg as AdminInstantiateMsg,
        ProposeAdminResp, QueryMsg as AdminQueryMsg,
    };
    use msgs::vote::{
        ExecuteMsg as VoteExecuteMsg, ProposedAdminResp, QueryMsg as VoteQueryMsg, VotesLeftResp,
    };

    use contract_admin::{
        execute as admin_execute, instantiate as admin_instantiate, query as admin_query,
        reply as admin_reply,
    };
    use contract_vote::{
        execute as vote_execute, instantiate as vote_instantiate, query as vote_query,
    };

    fn admin() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(admin_execute, admin_instantiate, admin_query)
            .with_reply(admin_reply);
        Box::new(contract)
    }

    fn vote() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(vote_execute, vote_instantiate, vote_query);
        Box::new(contract)
    }

    #[test]
    fn accept_vote() {
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("admin1"), coins(100, "utgd"))
                .unwrap();
        });
        let admin_code_id = app.store_code(admin());
        let vote_code_id = app.store_code(vote());

        let admin = app
            .instantiate_contract(
                admin_code_id,
                Addr::unchecked("owner"),
                &AdminInstantiateMsg {
                    admins: vec![
                        String::from("owner"),
                        String::from("admin1"),
                        String::from("admin2"),
                        String::from("admin3"),
                    ],
                    donation_denom: "eth".to_owned(),
                    vote_code_id,
                    quorum: Decimal::percent(75),
                },
                &[],
                "vote",
                None,
            )
            .unwrap();

        let resp: AppResponse = app
            .execute_contract(
                Addr::unchecked("owner"),
                admin,
                &AdminExecuteMsg::ProposeAdmin {
                    addr: String::from("new_admin"),
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let propose_admin_resp: ProposeAdminResp = from_binary(&resp.data.unwrap()).unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(
                propose_admin_resp.vote_addr.clone(),
                &VoteQueryMsg::VotesLeft {},
            )
            .unwrap();

        assert_eq!(
            resp,
            VotesLeftResp {
                votes_left: Decimal::from_str("3.0").unwrap()
            }
        );

        app.execute_contract(
            Addr::unchecked("admin1"),
            propose_admin_resp.vote_addr.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(
                propose_admin_resp.vote_addr.clone(),
                &VoteQueryMsg::VotesLeft {},
            )
            .unwrap();

        assert_eq!(
            resp,
            VotesLeftResp {
                votes_left: Decimal::from_str("2.0").unwrap()
            }
        );

        app.execute_contract(
            Addr::unchecked("admin1"),
            propose_admin_resp.vote_addr.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(
                propose_admin_resp.vote_addr.clone(),
                &VoteQueryMsg::VotesLeft {},
            )
            .unwrap();

        assert_eq!(
            resp,
            VotesLeftResp {
                votes_left: Decimal::from_str("2.0").unwrap()
            }
        );

        app.execute_contract(
            Addr::unchecked("admin2"),
            propose_admin_resp.vote_addr.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(
                propose_admin_resp.vote_addr.clone(),
                &VoteQueryMsg::VotesLeft {},
            )
            .unwrap();

        assert_eq!(
            resp,
            VotesLeftResp {
                votes_left: Decimal::from_str("1.0").unwrap()
            }
        );

        app.execute_contract(
            Addr::unchecked("admin3"),
            propose_admin_resp.vote_addr.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(
                propose_admin_resp.vote_addr.clone(),
                &VoteQueryMsg::VotesLeft {},
            )
            .unwrap();

        assert_eq!(
            resp,
            VotesLeftResp {
                votes_left: Decimal::zero()
            }
        );

        app.execute_contract(
            Addr::unchecked("admin3"),
            propose_admin_resp.vote_addr.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(propose_admin_resp.vote_addr, &VoteQueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(
            resp,
            VotesLeftResp {
                votes_left: Decimal::zero()
            }
        );
    }

    #[test]
    fn add_member() {
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("admin1"), coins(100, "utgd"))
                .unwrap();
        });
        let admin_code_id = app.store_code(admin());
        let vote_code_id = app.store_code(vote());

        let admin = app
            .instantiate_contract(
                admin_code_id,
                Addr::unchecked("owner"),
                &AdminInstantiateMsg {
                    admins: vec![String::from("owner"), String::from("admin1")],
                    donation_denom: "eth".to_owned(),
                    vote_code_id,
                    quorum: Decimal::percent(50),
                },
                &[],
                "vote",
                None,
            )
            .unwrap();

        let resp: AppResponse = app
            .execute_contract(
                Addr::unchecked("owner"),
                admin.clone(),
                &AdminExecuteMsg::ProposeAdmin {
                    addr: String::from("new_admin"),
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let propose_admin_resp: ProposeAdminResp = from_binary(&resp.data.unwrap()).unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 2);

        app.execute_contract(
            Addr::unchecked("admin1"),
            propose_admin_resp.vote_addr,
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin, &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 3);
    }

    #[test]
    fn duplicated_votes() {
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("admin1"), coins(100, "utgd"))
                .unwrap();
        });
        let admin_code_id = app.store_code(admin());
        let vote_code_id = app.store_code(vote());

        let admin = app
            .instantiate_contract(
                admin_code_id,
                Addr::unchecked("owner"),
                &AdminInstantiateMsg {
                    admins: vec![
                        "owner".to_owned(),
                        "admin1".to_owned(),
                        "admin2".to_owned(),
                        "admin3".to_owned(),
                    ],
                    donation_denom: "eth".to_owned(),
                    vote_code_id,
                    quorum: Decimal::percent(75),
                },
                &[],
                "vote",
                None,
            )
            .unwrap();

        let resp: AppResponse = app
            .execute_contract(
                Addr::unchecked("owner"),
                admin.clone(),
                &AdminExecuteMsg::ProposeAdmin {
                    addr: String::from("new_admin"),
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let propose_admin_resp: ProposeAdminResp = from_binary(&resp.data.unwrap()).unwrap();

        app.execute_contract(
            Addr::unchecked("admin1"),
            propose_admin_resp.vote_addr.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 4);

        app.execute_contract(
            Addr::unchecked("admin1"),
            propose_admin_resp.vote_addr.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 4);

        app.execute_contract(
            Addr::unchecked("admin2"),
            propose_admin_resp.vote_addr.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 4);

        app.execute_contract(
            Addr::unchecked("admin1"),
            propose_admin_resp.vote_addr.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 4);

        app.execute_contract(
            Addr::unchecked("admin3"),
            propose_admin_resp.vote_addr,
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin, &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 5);
    }

    #[test]
    fn accepting_vote_older_than_admin() {
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("admin1"), coins(100, "utgd"))
                .unwrap();
        });
        let admin_code_id = app.store_code(admin());
        let vote_code_id = app.store_code(vote());

        let admin = app
            .instantiate_contract(
                admin_code_id,
                Addr::unchecked("owner"),
                &AdminInstantiateMsg {
                    admins: vec![String::from("owner"), String::from("admin1")],
                    donation_denom: "eth".to_owned(),
                    vote_code_id,
                    quorum: Decimal::percent(50),
                },
                &[],
                "vote",
                None,
            )
            .unwrap();

        let resp: AppResponse = app
            .execute_contract(
                Addr::unchecked("owner"),
                admin.clone(),
                &AdminExecuteMsg::ProposeAdmin {
                    addr: String::from("admin2"),
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let propose_admin_2_resp: ProposeAdminResp = from_binary(&resp.data.unwrap()).unwrap();

        let resp: AppResponse = app
            .execute_contract(
                Addr::unchecked("owner"),
                admin.clone(),
                &AdminExecuteMsg::ProposeAdmin {
                    addr: String::from("admin3"),
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let propose_admin_3_resp: ProposeAdminResp = from_binary(&resp.data.unwrap()).unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 2);

        app.update_block(next_block);

        app.execute_contract(
            Addr::unchecked("admin1"),
            propose_admin_2_resp.vote_addr,
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 3);

        let err = app
            .execute_contract(
                Addr::unchecked("admin2"),
                propose_admin_3_resp.vote_addr.clone(),
                &VoteExecuteMsg::Accept {},
                &[],
            )
            .unwrap_err();

        assert_eq!(
            StdError::generic_err(
                "Admin is not allowed to vote due to being approved after vote is created."
            ),
            err.downcast().unwrap()
        );

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 3);

        app.execute_contract(
            Addr::unchecked("admin1"),
            propose_admin_3_resp.vote_addr,
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin, &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 4);
    }

    #[test]
    fn required_votes_not_being_integers() {
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("admin1"), coins(100, "utgd"))
                .unwrap();
        });
        let admin_code_id = app.store_code(admin());
        let vote_code_id = app.store_code(vote());

        let admin = app
            .instantiate_contract(
                admin_code_id,
                Addr::unchecked("owner"),
                &AdminInstantiateMsg {
                    admins: vec![
                        String::from("owner"),
                        String::from("admin1"),
                        String::from("admin2"),
                    ],
                    donation_denom: "eth".to_owned(),
                    vote_code_id,
                    quorum: Decimal::percent(40),
                },
                &[],
                "vote",
                None,
            )
            .unwrap();

        let resp: AppResponse = app
            .execute_contract(
                Addr::unchecked("owner"),
                admin.clone(),
                &AdminExecuteMsg::ProposeAdmin {
                    addr: String::from("admin3"),
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let resp: ProposeAdminResp = from_binary(&resp.data.unwrap()).unwrap();
        let vote_addr = resp.vote_addr;

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 3);

        app.execute_contract(
            Addr::unchecked("admin1"),
            vote_addr,
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin, &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 4);
    }

    #[test]
    fn vote_instantiation() {
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("admin1"), coins(100, "utgd"))
                .unwrap();
        });
        let admin_code_id = app.store_code(admin());
        let vote_code_id = app.store_code(vote());

        let admin = app
            .instantiate_contract(
                admin_code_id,
                Addr::unchecked("owner"),
                &AdminInstantiateMsg {
                    admins: vec![String::from("owner"), String::from("admin1")],
                    donation_denom: "eth".to_owned(),
                    vote_code_id,
                    quorum: Decimal::percent(50),
                },
                &[],
                "vote",
                None,
            )
            .unwrap();

        let resp: AppResponse = app
            .execute_contract(
                Addr::unchecked("owner"),
                admin,
                &AdminExecuteMsg::ProposeAdmin {
                    addr: String::from("proposed_admin"),
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let resp: ProposeAdminResp = from_binary(&resp.data.unwrap()).unwrap();
        let addr = resp.vote_addr;

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &VoteQueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(
            resp,
            VotesLeftResp {
                votes_left: Decimal::from_str("1.0").unwrap()
            }
        );

        let resp: ProposedAdminResp = app
            .wrap()
            .query_wasm_smart(addr, &VoteQueryMsg::ProposedAdmin {})
            .unwrap();

        assert_eq!(
            resp,
            ProposedAdminResp {
                proposed_admin: Addr::unchecked("proposed_admin")
            }
        );
    }
}
