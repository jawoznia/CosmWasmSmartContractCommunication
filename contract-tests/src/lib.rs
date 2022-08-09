#[cfg(test)]
mod tests {
    use contract_msgs::admin::{ExecuteMsg, InstantiateMsg as AdminInstantiateMsg};
    use contract_msgs::vote::{ExecuteMsg as VoteExecuteMsg, QueryMsg, VotesLeftResp};
    use cosmwasm_std::{coins, from_binary, Addr, Empty};
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

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
                &ExecuteMsg::ProposeAdmin {
                    addr: Addr::unchecked("new_admin"),
                    required_votes: 3,
                    admin_code_id,
                },
                &[],
            )
            .unwrap();

        let vote: Addr = from_binary(&resp.data.unwrap()).unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(vote.clone(), &QueryMsg::VotesLeft {})
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
            .query_wasm_smart(vote.clone(), &QueryMsg::VotesLeft {})
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
            .query_wasm_smart(vote.clone(), &QueryMsg::VotesLeft {})
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
            .query_wasm_smart(vote.clone(), &QueryMsg::VotesLeft {})
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
            .query_wasm_smart(vote.clone(), &QueryMsg::VotesLeft {})
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
            .query_wasm_smart(vote.clone(), &QueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 0 });

        // app.execute_contract(
        //     Addr::unchecked("member"),
        //     manager.clone(),
        //     &ExecMsg::Join {},
        //     &[],
        // )
        // .unwrap();

        // let peer: MemberPeerAddrResp = app
        //     .wrap()
        //     .query_wasm_smart(
        //         manager,
        //         &QueryMsg::MemberPeerAddr {
        //             addr: "member".to_owned(),
        //         },
        //     )
        //     .unwrap();

        // app.execute_contract(
        //     Addr::unchecked("donator"),
        //     peer.addr.clone(),
        //     &PeerExec::Donate {},
        //     &coins(100, "utgd"),
        // )
        // .unwrap();

        // app.execute_contract(
        //     Addr::unchecked("member"),
        //     peer.addr.clone(),
        //     &PeerExec::Withdraw {},
        //     &[],
        // )
        // .unwrap();

        // assert_eq!(
        //     coin(0, "utgd"),
        //     app.wrap().query_balance("donator", "utgd").unwrap()
        // );
        // assert_eq!(
        //     coin(0, "utgd"),
        //     app.wrap()
        //         .query_balance(peer.addr.as_str(), "utgd")
        //         .unwrap()
        // );
        // assert_eq!(
        //     coin(100, "utgd"),
        //     app.wrap().query_balance("member", "utgd").unwrap()
        // );

        // #[test]
        // fn accept_vote() {
        //     let mut app = App::default();

        //     let admin_code = admin();
        //     let admin_code_id = app.store_code(admin_code);

        //     let vote_code = vote();
        //     let vote_code_id = app.store_code(vote_code);

        //     let addr = app
        //         .instantiate_contract(
        //             vote_code_id,
        //             Addr::unchecked("owner"),
        //             &InstantiateMsg {
        //                 proposed_admin: Addr::unchecked("new_admin"),
        //                 required: 3,
        //             },
        //             &[],
        //             "Contract",
        //             None,
        //         )
        //         .unwrap();

        //     let resp: VotesLeftResp = app
        //         .wrap()
        //         .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
        //         .unwrap();

        //     assert_eq!(resp, VotesLeftResp { votes_left: 3 });

        //     app.execute_contract(Addr::unchecked("admin1"), addr.clone(), &VoteExecuteMsg::Accept {}, &[])
        //         .unwrap();

        //     let resp: VotesLeftResp = app
        //         .wrap()
        //         .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
        //         .unwrap();

        //     assert_eq!(resp, VotesLeftResp { votes_left: 2 });

        //     app.execute_contract(Addr::unchecked("admin1"), addr.clone(), &VoteExecuteMsg::Accept {}, &[])
        //         .unwrap();

        //     let resp: VotesLeftResp = app
        //         .wrap()
        //         .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
        //         .unwrap();

        //     assert_eq!(resp, VotesLeftResp { votes_left: 2 });

        //     app.execute_contract(Addr::unchecked("admin2"), addr.clone(), &VoteExecuteMsg::Accept {}, &[])
        //         .unwrap();

        //     let resp: VotesLeftResp = app
        //         .wrap()
        //         .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
        //         .unwrap();

        //     assert_eq!(resp, VotesLeftResp { votes_left: 1 });

        //     app.execute_contract(Addr::unchecked("admin3"), addr.clone(), &VoteExecuteMsg::Accept {}, &[])
        //         .unwrap();

        //     let resp: VotesLeftResp = app
        //         .wrap()
        //         .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
        //         .unwrap();

        //     assert_eq!(resp, VotesLeftResp { votes_left: 0 });

        //     app.execute_contract(Addr::unchecked("admin3"), addr.clone(), &VoteExecuteMsg::Accept {}, &[])
        //         .unwrap();

        //     let resp: VotesLeftResp = app
        //         .wrap()
        //         .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
        //         .unwrap();

        //     assert_eq!(resp, VotesLeftResp { votes_left: 0 });
    }
}
