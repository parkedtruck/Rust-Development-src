

#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Recipient, State, UnparsedRecipient, Will, MEMBERSHIPS, STATE, WILLS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:will";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_RENEWAL_RATE: u128 = 60 * 60 * 24 * 365; 

// ---------------------------------------------------------------------------
// Iniatialization
// ---------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        admin: info.sender.clone(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner_address", info.sender.clone()))
}

// ---------------------------------------------------------------------------
// Mutation
// ---------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ResetTimestamp {} => try_reset_timestamp(deps, env, info),
        ExecuteMsg::DepositTokens {} => try_deposit_tokens(deps, env, info),
        ExecuteMsg::WithdrawTokens { tokens } => try_withdraw_tokens(deps, env, info, tokens),
        ExecuteMsg::DistributeAssets { owner } => try_distribute_assets(deps, env, info, owner),
        ExecuteMsg::SetRecipients { recipients } => try_set_recipients(deps, env, info, recipients),
        ExecuteMsg::SetRenewalRate { renewal_rate } => try_set_renewal_rate(deps, env, info, renewal_rate)
    }
}

pub fn try_set_renewal_rate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_renewal_rate: Uint128,
) -> Result<Response, ContractError> {
    let mut will = match WILLS.load(deps.storage, info.sender.clone()) {
        Ok(will) => will,
        Err(_) => Will {
            owner: info.sender.clone(),
            recipients: Vec::new(),
            timestamp: Uint128::from(env.block.time.seconds()),
            renewal_rate: Uint128::from(DEFAULT_RENEWAL_RATE),
            tokens: Vec::new(),
        },
    }
    .clone();

    will.renewal_rate = new_renewal_rate;
    will.timestamp = Uint128::from(env.block.time.seconds());
    WILLS.save(deps.storage, info.sender, &will)?;

    Ok(Response::new().add_attribute("action", "set_renewal_rate"))
}

pub fn try_reset_timestamp(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut will = match WILLS.load(deps.storage, info.sender.clone()) {
        Ok(will) => will,
        Err(_) => Will {
            owner: info.sender.clone(),
            recipients: Vec::new(),
            timestamp: Uint128::from(env.block.time.seconds()),
            renewal_rate: Uint128::from(DEFAULT_RENEWAL_RATE),
            tokens: Vec::new(),
        },
    }
    .clone();

    will.timestamp = Uint128::from(env.block.time.seconds());
    WILLS.save(deps.storage, info.sender, &will)?;

    Ok(Response::new().add_attribute("action", "reset_timestamp"))
}

pub fn try_set_recipients(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_recipients: Vec<UnparsedRecipient>,
) -> Result<Response, ContractError> {
    let mut will = match WILLS.load(deps.storage, info.sender.clone()) {
        Ok(will) => will,
        Err(_) => Will {
            owner: info.sender.clone(),
            recipients: Vec::new(),
            timestamp: Uint128::from(env.block.time.seconds()),
            renewal_rate: Uint128::from(DEFAULT_RENEWAL_RATE),
            tokens: Vec::new(),
        },
    }
    .clone();

    let mut new_parsed_recipients = Vec::new();
    for recipient in &new_recipients.clone() {
        let parsed_recipient = Recipient {
            address: deps.api.addr_validate(&recipient.address.to_string())?,
            percentage: recipient.percentage,
        };
        new_parsed_recipients.push(parsed_recipient);
    }

    let mut sum = Uint128::from(0u64); //check that sum of tokens = 100 and validates addrs
    let mut parsed_recipients = Vec::new();
    for recipient in &new_recipients.clone() {
        sum = sum.checked_add(recipient.percentage).unwrap();
        let validated_addr = deps
            .api
            .addr_validate(&recipient.address.to_string())
            .unwrap();
        parsed_recipients.push(Recipient {
            address: validated_addr,
            percentage: recipient.percentage,
        });
    }

    if sum != Uint128::from(100u64) {
        return Err(ContractError::InvalidRecipients {});
    }

    for new_recipient in parsed_recipients {
        //in vector of recipients they are passing
        let mut found = false;
        for old_recipient in will.recipients.iter_mut() {
            //in vector of old recipients
            if old_recipient.address == new_recipient.address {
                old_recipient.percentage = new_recipient.percentage; //update percentage
                found = true;
            }
        }
        if !found {
            will.recipients.push(Recipient {
                address: new_recipient.clone().address,
                percentage: new_recipient.clone().percentage,
            });
            match MEMBERSHIPS.load(deps.storage, new_recipient.clone().address) {
                Ok(wills) => {
                    let mut wills = wills.clone();
                    wills.push(info.sender.clone());
                    MEMBERSHIPS.save(deps.storage, new_recipient.clone().address, &wills)?;
                }
                Err(_) => {
                    let mut wills = Vec::new();
                    wills.push(info.sender.clone());
                    MEMBERSHIPS.save(deps.storage, new_recipient.clone().address, &wills)?;
                }
            };
        }
    }

    will.timestamp = Uint128::from(env.block.time.seconds());
    WILLS.save(deps.storage, info.sender.clone(), &will)?;

    Ok(Response::new().add_attribute("action", "set_recipients"))
}

pub fn try_deposit_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut will = match WILLS.load(deps.storage, info.sender.clone()) {
        Ok(will) => will,
        Err(_) => Will {
            owner: info.sender.clone(),
            recipients: Vec::new(),
            timestamp: Uint128::from(env.block.time.seconds()),
            renewal_rate: Uint128::from(DEFAULT_RENEWAL_RATE),
            tokens: Vec::new(),
        },
    }
    .clone();

    for token in &info.funds {
        if token.denom == "cw20" {
            return Err(ContractError::InvalidTokenDenom {});
        }

        let token_index = will.tokens.iter().position(|x| x.denom == token.denom);
        if token_index == None {
            will.tokens.push(token.clone());
        } else {
            will.tokens[token_index.unwrap()].amount += token.amount;
        }
    }

    will.timestamp = Uint128::from(env.block.time.seconds());
    WILLS.save(deps.storage, info.sender, &will)?;

    Ok(Response::new().add_attribute("action", "deposit_tokens"))
}

pub fn try_withdraw_tokens(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    tokens: Vec<Coin>,
) -> Result<Response, ContractError> {
    let mut will = match WILLS.load(deps.storage, info.sender.clone()) {
        Ok(will) => will,
        Err(_) => return Err(ContractError::NonExistentWill {}),
    }
    .clone();

    //check prev total vs removed
    for token in &tokens {
        if token.denom == "cw20" {
            return Err(ContractError::InvalidTokenDenom {});
        }
        let token_index = will.tokens.iter().position(|x| x.denom == token.denom);
        if token_index == None {
            return Err(ContractError::InsufficientFunds {});
        } else {
            if will.tokens[token_index.unwrap()].amount < token.amount {
                return Err(ContractError::InsufficientFunds {});
            }
            let new_amount = will.tokens[token_index.unwrap()]
                .amount
                .checked_sub(token.amount)
                .unwrap();
            will.tokens[token_index.unwrap()].amount = new_amount;
        }
    }
    WILLS.save(deps.storage, info.sender.clone(), &will)?;

    Ok(Response::new()
        .add_attribute("action", "withdraw_tokens")
        .add_message(BankMsg::Send {
            to_address: info.sender.clone().into(),
            amount: tokens,
        }))
}

pub fn try_distribute_assets(
    deps: DepsMut,
    env: Env,
    _: MessageInfo,
    owner: String,
) -> Result<Response, ContractError> {
    let parsed_addr = deps.api.addr_validate(&owner)?;

    let will = match WILLS.load(deps.storage, parsed_addr.clone()) {
        Ok(will) => will,
        Err(_) => return Err(ContractError::NonExistentWill {}),
    }
    .clone();

    let elapsed_time = Uint128::from(env.block.time.seconds()) - will.timestamp;

    if elapsed_time < will.renewal_rate {
        return Err(ContractError::NotClaimable {});
    }

    //make sure percentage sharei s above a certain amt uLuna
    //or we could not let ppl assign a percent to someone thats below 10000 uluna
    /*
    //FEE TAKING
    let mut fee = Vec::new();
    for token in &will.tokens {
        let token_fee = token
            .amount
            .clone()
            .checked_div(Uint128::from(1000u128))
            .unwrap();
        if token_fee >= Uint128::from(1000u128) {
            fee.push(Coin {
                denom: token.denom.clone(),
                amount: token_fee
            })
        }
        if token_fee == Uint128::from(0u128) {
        }
    }
    let fee_transaction = BankMsg::Send {
        to_address: owner_address.into(),
        amount: fee,
    };
    */
    
    let mut distribute_transactions = Vec::new();

    for recipient in &will.recipients {
        let mut valid_tokens : Vec<Coin> = Vec::new();

        for token in &will.tokens {
            let token_share = token
                .amount
                .clone()
                .checked_mul(recipient.percentage)
                .unwrap()
                .checked_div(Uint128::from(100u128))
                .unwrap();

            if token_share >= Uint128::from(100000u128) {//TODO: ask abt finding min txn amt
                valid_tokens.push(Coin {
                    denom: token.denom.clone(),
                    amount: token_share,
                });
            }
        }
        if valid_tokens.len() > 0 {
            distribute_transactions.push(BankMsg::Send{
                to_address: recipient.clone().address.into(),
                amount: valid_tokens,
            })
        }
    }

    WILLS.remove(deps.storage, parsed_addr.clone());
    

    for recipient in &will.recipients {
        match MEMBERSHIPS.load(deps.storage, recipient.clone().address) {
            Ok(wills) => {
                let mut wills = wills.clone();
                wills.retain(|owner| *owner != parsed_addr.clone());
                MEMBERSHIPS.save(deps.storage, recipient.clone().address, &wills)?;
            }
            Err(_) => {
                let wills = Vec::new();
                MEMBERSHIPS.save(deps.storage, recipient.clone().address, &wills)?;
            }
        }
    }

    Ok(Response::new()
        .add_attribute("method", "distribute_assets")
        .add_messages(distribute_transactions))
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetWill { address } => to_binary(&try_get_will(deps, env, address)?),
        QueryMsg::SeeMemberships { address } => {
            to_binary(&try_see_memberships(deps, env, address)?)
        }
    }
}

pub fn try_get_will(deps: Deps, env: Env, address: String) -> StdResult<Will> {
    let parsed_addr = deps.api.addr_validate(&address)?;
    let will = match WILLS.load(deps.storage, parsed_addr.clone()) {
        Ok(will) => will,
        Err(_) => Will {
            owner: parsed_addr,
            recipients: Vec::new(),
            timestamp: Uint128::from(env.block.time.seconds()),
            renewal_rate: Uint128::from(DEFAULT_RENEWAL_RATE),
            tokens: Vec::new(),
        },
    };

    Ok(will)
}

pub fn try_see_memberships(deps: Deps, env: Env, address: String) -> StdResult<Vec<Will>> {
    let parsed_addr = deps.api.addr_validate(&address)?;
    let mut wills = Vec::new();
    let memberships = match MEMBERSHIPS.load(deps.storage, parsed_addr) {
        Ok(membership) => membership,
        Err(_) => Vec::new(),
    };
    for member in memberships {
        let will = match WILLS.load(deps.storage, member.clone()) {
            Ok(will) => will,
            Err(_) => Will {
                owner: member,
                recipients: Vec::new(),
                timestamp: Uint128::from(env.block.time.seconds()),
                renewal_rate: Uint128::from(DEFAULT_RENEWAL_RATE),
                tokens: Vec::new(),
            },
        };
        wills.push(will);
    }
    Ok(wills)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Uint128};

    #[test]
    fn init() {
        let addr = "creator";
        let mut deps = mock_dependencies(&[]);
        let info = mock_info(addr, &coins(2, "uluna"));

        instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg {}).unwrap();
    }

    #[test]
    fn depositAndWithdraw() {
        let addr = "creator";
        let mut deps = mock_dependencies(&[]);
        let info = mock_info(addr, &coins(2, "uluna"));
        let mut recipients = Vec::new();
        recipients.push(UnparsedRecipient {
            address: addr.to_string(),
            percentage: Uint128::from(100 as u64),
        });
        let mut tokens = Vec::new();
        tokens.push(Coin {
            denom: "uluna".to_string(),
            amount: Uint128::from(2 as u128),
        });

        try_deposit_tokens(deps.as_mut(), mock_env(), info.clone()).unwrap();

        let willAfterDeposit = try_get_will(deps.as_ref(), mock_env(), addr.to_string()).unwrap();

        assert_eq!(willAfterDeposit.recipients.len(), 1);
        assert_eq!(
            willAfterDeposit.tokens[0],
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(2 as u128),
            }
        );
        assert_eq!(willAfterDeposit.recipients[0].address, addr.to_string());
        assert_eq!(willAfterDeposit.recipients[0].percentage, 100);

        try_withdraw_tokens(deps.as_mut(), mock_env(), info.clone(), tokens.clone()).unwrap();

        let willAfterWithdraw = try_get_will(deps.as_ref(), mock_env(), addr.to_string()).unwrap();
        assert_eq!(
            willAfterWithdraw.tokens[0],
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(0 as u128),
            }
        );
    }
}
