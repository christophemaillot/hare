use crate::harehandler::{HareError, HareHandler};

mod harehandler;
mod amqputils;

#[tokio::main]
async fn main() -> Result<(), HareError> {

    let hare = HareHandler::new();
    hare.start().await?;

    Ok(())
}
