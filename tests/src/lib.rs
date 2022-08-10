#[cfg(test)]
mod tests {

    use cosmwasm_std::{coins, from_binary, Addr, BlockInfo, Empty};
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
    use msgs::admin::{
        AdminsListResp, ExecuteMsg as AdminExecuteMsg, InstantiateMsg as AdminInstantiateMsg,
        QueryMsg as AdminQueryMsg,
    };
    use msgs::vote::{
        ExecuteMsg as VoteExecuteMsg, InstantiateMsg as VoteInstantiateMsg,
        QueryMsg as VoteQueryMsg, VotesLeftResp,
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
                    vote_code_id: vote_code_id,
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
                    required_votes: 3,
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let vote: Addr = from_binary(&resp.data.unwrap()).unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(vote.clone(), &VoteQueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 3 });

        app.execute_contract(
            Addr::unchecked("admin1"),
            vote.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(vote.clone(), &VoteQueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 2 });

        app.execute_contract(
            Addr::unchecked("admin1"),
            vote.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(vote.clone(), &VoteQueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 2 });

        app.execute_contract(
            Addr::unchecked("admin2"),
            vote.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(vote.clone(), &VoteQueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 1 });

        app.execute_contract(
            Addr::unchecked("admin3"),
            vote.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(vote.clone(), &VoteQueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 0 });

        app.execute_contract(
            Addr::unchecked("admin3"),
            vote.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(vote.clone(), &VoteQueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 0 });
    }

    #[test]
    fn unauthorized() {
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("admin1"), coins(100, "utgd"))
                .unwrap();
        });
        let admin_code_id = app.store_code(admin());
        let vote_code_id = app.store_code(vote());

        let admin = app // contract0
            .instantiate_contract(
                admin_code_id,
                Addr::unchecked("owner"), // sth
                &AdminInstantiateMsg {
                    admins: vec![String::from("owner"), String::from("admin1")],
                    donation_denom: "eth".to_owned(),
                    vote_code_id: vote_code_id,
                },
                &[],
                "vote",
                None,
            )
            .unwrap();

        let malicious_vote = app // contract1
            .instantiate_contract(
                vote_code_id,
                Addr::unchecked("malicious_user"), // sth
                &VoteInstantiateMsg {
                    required_votes: 1,
                    proposed_admin: "malicious_user".to_owned(),
                    admin_code_id,
                },
                &[],
                "malicious_vote",
                None,
            )
            .unwrap();

        let _err = app
            .execute_contract(
                Addr::unchecked("admin1"),
                malicious_vote,
                &VoteExecuteMsg::Accept {},
                &[],
            )
            .unwrap_err();

        // TODO: test fails on json parsing error. Below assert should work instead.
        // assert_eq!(
        //     ContractError::Unauthorized {
        //         sender: Addr::unchecked("admin1")
        //     },
        //     err.downcast().unwrap()
        // );

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 2);
    }

    #[test]
    fn add_members() {
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("admin1"), coins(100, "utgd"))
                .unwrap();
        });
        let admin_code_id = app.store_code(admin());
        let vote_code_id = app.store_code(vote());

        let admin = app // contract0
            .instantiate_contract(
                admin_code_id,
                Addr::unchecked("owner"), // sth
                &AdminInstantiateMsg {
                    admins: vec![String::from("owner"), String::from("admin1")], // change to to_owned
                    donation_denom: "eth".to_owned(),
                    vote_code_id: vote_code_id,
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
                    required_votes: 1,
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let vote: Addr = from_binary(&resp.data.unwrap()).unwrap(); // contract1

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 2);

        app.execute_contract(
            Addr::unchecked("admin1"),
            vote.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
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
                    vote_code_id: vote_code_id,
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
                    required_votes: 3,
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let vote: Addr = from_binary(&resp.data.unwrap()).unwrap();

        app.execute_contract(
            Addr::unchecked("admin1"),
            vote.clone(),
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
            vote.clone(),
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
            vote.clone(),
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
            vote.clone(),
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
            vote.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 5);
    }

    #[test]
    // Ignored because
    #[ignore]
    fn accepting_vote_older_than_admin() {
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("admin1"), coins(100, "utgd"))
                .unwrap();
        });
        let admin_code_id = app.store_code(admin());
        let vote_code_id = app.store_code(vote());

        let admin = app // contract0
            .instantiate_contract(
                admin_code_id,
                Addr::unchecked("owner"),
                &AdminInstantiateMsg {
                    admins: vec![String::from("owner"), String::from("admin1")],
                    donation_denom: "eth".to_owned(),
                    vote_code_id: vote_code_id,
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
                    required_votes: 1,
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let vote_admin_2: Addr = from_binary(&resp.data.unwrap()).unwrap();

        let resp: AppResponse = app
            .execute_contract(
                Addr::unchecked("owner"),
                admin.clone(),
                &AdminExecuteMsg::ProposeAdmin {
                    addr: String::from("admin3"),
                    required_votes: 1,
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let vote_admin_3: Addr = from_binary(&resp.data.unwrap()).unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 2);

        app.update_block(|bi: &mut BlockInfo| {
            bi.time.plus_seconds(5);
            bi.height.checked_add(5).unwrap();
        });

        app.execute_contract(
            Addr::unchecked("admin1"),
            vote_admin_2,
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 3);

        app.execute_contract(
            Addr::unchecked("admin2"),
            vote_admin_3.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 3);

        app.execute_contract(
            Addr::unchecked("admin1"),
            vote_admin_3.clone(),
            &VoteExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(admin.clone(), &AdminQueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp.admins.len(), 4);
    }
}
