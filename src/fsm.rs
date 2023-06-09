use crate::error::BotError;
use aws_sdk_dynamodb::{
    types::{AttributeValue, Select},
    Client,
};
use std::collections::HashMap;

pub struct FSMClient {
    client: Client,
    table_name: String,
}

impl FSMClient {
    pub async fn build(table_name: String) -> FSMClient {
        let shared_config = aws_config::load_from_env().await;
        let client = Client::new(&shared_config);

        Self { client, table_name }
    }

    pub async fn reset(&self, chat_id: i64) -> Result<(), BotError> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("chat_id", AttributeValue::N(chat_id.to_string()))
            .send()
            .await
            .map_err(|_| BotError::FsmError(String::from("FSM store is not available")))?;

        Ok(())
    }

    pub async fn get_state(&self, chat_id: i64) -> Result<Option<String>, BotError> {
        let response = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression(String::from("#key = :value"))
            .expression_attribute_names(String::from("#key"), String::from("chat_id"))
            .expression_attribute_values(
                String::from(":value"),
                AttributeValue::N(chat_id.to_string()),
            )
            .select(Select::AllAttributes)
            .send()
            .await
            .map_err(|_| BotError::FsmError(String::from("FSM store is not available")))?;

        let raw_state_map = response
            .items()
            .ok_or(BotError::ParsingError(String::from(
                "Given state is not recorded in FSM store",
            )))?
            .first();

        let state = raw_state_map
            .and_then(|m| m.get("state"))
            .and_then(|r| r.as_s().ok().cloned());

        Ok(state)
    }

    pub async fn set_state(&self, chat_id: i64, state_name: String) -> Result<(), BotError> {
        let mut state_metadata: HashMap<String, AttributeValue> = HashMap::new();

        state_metadata.insert(String::from("scores"), AttributeValue::M(HashMap::new()));

        let request = self
            .client
            .put_item()
            .table_name(&self.table_name)
            .item("chat_id", AttributeValue::N(chat_id.to_string()))
            .item("state", AttributeValue::S(state_name))
            .item("meta", AttributeValue::M(state_metadata));

        request
            .send()
            .await
            .map_err(|_| BotError::FsmError(String::from("FSM store is not available")))?;

        Ok(())
    }
}
