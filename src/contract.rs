#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo,Addr, Response,  StdResult, BankMsg, Decimal, Uint128, Coin, SubMsg}; //Uint128, SubMsg, CosmosMsg
use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetFeePercentageResponse, InstantiateMsg, QueryMsg, GetFeeReceiverAddResponse};
use crate::state::{State, STATE};


const CONTRACT_NAME: &str = "crates.io:{{project-name}}";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let state = State {
        fee: _msg.fee,
        fee_receiver: _msg.fee_receiver,
        owner: _info.sender.clone(),
    };
    set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(_deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", &_info.sender.to_string())
        .add_attribute("fee", _msg.fee.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateFee { fee } => try_update_transaction_fee(deps, info, fee),
        ExecuteMsg::UpdateFeeReceiver{ fee_receiver } => try_update_fee_receiver(deps, info, fee_receiver.to_string()),
        ExecuteMsg::Transfer { to } => execute_transfer(deps, info, to.to_string())
    }
}

// #[cfg_attr(not(feature = "library"), entry_point)]
pub fn try_update_transaction_fee(deps: DepsMut, info: MessageInfo, fee: Decimal) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.fee = fee;
        Ok(state)
    })?;
    Ok(Response::new()
    .add_attribute("method", "update_transaction_fee")
    .add_attribute("new_transaction_fee", fee.to_string())
)
}

pub fn try_update_fee_receiver(deps:DepsMut, info: MessageInfo, fee_receiver: String) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized{});
        }
        state.fee_receiver = Addr::unchecked(fee_receiver);
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method","updated fee receiver address"))
}

pub fn execute_transfer(deps:DepsMut, mut _info:MessageInfo, recipient: String)-> Result<Response, ContractError> {
   
    // if _info.funds.is_empty() {
    //    return Err(ContractError::InvalidZeroAmount {});
    // }
    // if _info.funds.iter().all(|c| c.amount == Uint128::zero()) {
    //     return Err(ContractError::InvalidZeroAmount {});
    // } 
    if _info.funds[0].amount == Uint128::zero(){
        return Err(ContractError::InvalidZeroAmount {});
    } else {
        let state = STATE.load(deps.storage)?;

        let sto: Uint128 = Uint128::new(100);

        // let _fee = state.fee.atomics();
        let _fee = Uint128::new(1);

        
        // let fee_coins: Vec<_> = _info.funds
        //         .iter()
        //         .cloned()
        //         .map(|mut coin| {
        //             coin.amount = Uint128::new(coin.amount.u128() * _fee.u128() / sto.u128());
        //             coin
        //         })
        //         .collect();

        // let transaction_sum: Uint128 = fee_coins
        //     .iter()
        //     .cloned()
        //     .map(|coin| {
        //         coin.amount
        //     }).sum();

        // let transfer_coin: Vec<_> = _info.funds
        //         .iter()
        //         .cloned()
        //         .map(|mut coin| {
        //             coin.amount = Uint128::new(coin.amount.u128() - transaction_sum.u128());
        //             coin
        //         })
        //         .collect();
        
        let to = deps.api.addr_validate(&recipient)?;

        let b_tx: Uint128 = Uint128::new((_info.funds[0].amount.u128() * _fee.u128()) / sto.u128());


        _info.funds[0].amount = Uint128::new(_info.funds[0].amount.u128() - b_tx.u128());


        let denom = _info.funds[0].denom.to_string();
        if b_tx.is_zero() && _info.funds[0].amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        } else {
            let send_to_comission = BankMsg::Send {
                to_address: state.fee_receiver.to_string(),
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount: b_tx,
                }] 
            };
            let send_to_receiver = BankMsg::Send {
                to_address: to.to_string(),
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount: _info.funds[0].amount,
                }] 
            };  
            // let fee_transaction = BankMsg::Send {   
            //     to_address: state.fee_receiver.clone().into(),
            //     amount: fee_coins.clone(),
            // };
            // let transfer_transaction = BankMsg::Send {
            //     to_address: to.clone().into(),
            //     amount: _info.funds.clone(),
            // };
            const ID2: u64 = 2;
            Ok(
                Response::new()
                // .add_messages(vec![CosmosMsg::Bank(send_to_comission), CosmosMsg::Bank(send_to_receiver)])
                .add_submessage(SubMsg::new(send_to_comission))
                .add_message(send_to_receiver)
                // .add_messages(vec![send_to_comission, send_to_receiver])
                .add_attribute("action", "transfer")
    
            )
        }
       
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetFeePercentage {} => to_binary(&query_fee_percentage(deps)?),
        QueryMsg::GetFeeReceiverAdd {} => to_binary(&query_fee_receiver(deps)?),
     }
}

fn query_fee_percentage(deps: Deps) -> StdResult<GetFeePercentageResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(GetFeePercentageResponse { fee: state.fee })
}

fn query_fee_receiver(deps: Deps) -> StdResult<GetFeeReceiverAddResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(GetFeeReceiverAddResponse {fee_receiver: state.fee_receiver})
}

// .add_message(CosmosMsg::Bank(transfer_transaction))
// fn send_tokens(to_address: Addr, amount: Vec<Coin>, action: &str) -> Response {
//     Response::new()
//         .add_message(CosmosMsg::Bank(BankMsg::Send {
//             to_address: to_address.clone().into(),
//             amount,
//         }))
//         .add_attribute("action", action)
//         .add_attribute("to", to_address)
// }

#[cfg(test)]
mod tests {

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let account = "wasm1ycf7rum00dkwvzh0clntxsdwxnq7hd7ngrj584";
        let msg = InstantiateMsg { fee: Decimal::one(), fee_receiver: Addr::unchecked(account)};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetFeePercentage {}).unwrap();
        let value: GetFeePercentageResponse = from_binary(&res).unwrap();
        assert_eq!(Decimal::one(), value.fee);
    }

    // #[test]
    // fn update_reciver() {
    //     let mut deps = mock_dependencies();

    //     let msg = UpdateFeeReceiver { fee: 1 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.fee);
    // }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies();

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
