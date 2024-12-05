use std::collections::HashMap;
use std::env::VarError;
use std::path::Path;
use std::time::SystemTime;
use futures_lite::StreamExt;
use lapin::options::BasicConsumeOptions;
use lapin::{options::*, types::FieldTable};
use lapin::message::Delivery;
use log::SetLoggerError;
use thiserror::Error;
use crate::{amqputils};

#[derive(Error, Debug)]
pub enum HareError {
    #[error("RabbitMQ issue error: {0}")]
    AmqpConnectionError(#[from] lapin::Error),

    #[error("script error: {0}")]
    ScriptError(#[from]std::io::Error),

    #[error("logger error: {0}")]
    LogConfigurationError(#[from] fern::InitError),

    #[error("logging initialization error: {0}")]
    LoggingInitError(#[from]SetLoggerError),
}

pub struct HareHandler {
    script_root: String,            // path to scripts root
    rabbitmq_url: String,           // rabbitmq url
    queue_name: String,             // queue name to listen on
    handler_key: String,            // header key to use for handler script name
    log_destination: Option<String> // filename to log to
}

impl HareHandler {

    /// Creates a new instance of the HareHandler.
    ///
    /// The constructor reads environment variables to configure the handler,
    /// or uses default values if the variables are not set.
    ///
    /// @return HareHandler
    ///
    pub fn new() -> Self {
        HareHandler {
            script_root: std::env::var("HARE_SCRIPT_ROOT").unwrap_or_else(|_| "/etc/hare/scripts".to_string()),
            rabbitmq_url: std::env::var("HARE_AMQP_URL").unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".to_string()),
            queue_name: std::env::var("HARE_AMQP_QUEUE").unwrap_or_else(|_| "deploy".to_string()),
            handler_key: std::env::var("HARE_HANDLER_KEY").unwrap_or_else(|_| "type".to_string()),

            log_destination: match std::env::var("HARE_LOG_DESTINATION") {
                Ok(value) => { Some(value)}
                Err(_) => { None }
            }
        }
    }
}

impl HareHandler {

    /// Start the hare handler.
    ///
    /// This function will connect to RabbitMQ, consume messages from the queue, and execute scripts based on the message type.
    /// It will also log any errors that occur during the execution of the scripts.
    ///
    /// @return Result<(), HareError>
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with the RabbitMQ connection or script execution.
    pub async fn start(&self) -> Result<(), HareError> {
        self.configure_logging();
        self.rabbitmq_loop().await?;
        Ok(())
    }

    /// Configures the logger based on the environment variables.
    ///
    /// Uses the `HARE_LOG_DESTINATION` variable to determine the log destination.
    /// If the variable is not set, the logger will log to the console.
    /// If the variable is set, the logger will log to the specified file.
    ///
    fn configure_logging(&self) -> Result<(), HareError> {
        let dispatch = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{} {} {}] {}",
                    humantime::format_rfc3339_seconds(SystemTime::now()),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .level(log::LevelFilter::Debug);

        let dispatch = match &self.log_destination {
            None => {
                dispatch.chain(std::io::stdout())
            }
            Some(path) => {
                dispatch.chain(fern::log_file(path)?)
            }
        };

        let result = dispatch.apply();
        Ok(())
    }

    /// RabbitMQ message consumer loop.
    ///
    /// This function connects to RabbitMQ, creates a channel, and consumes messages from the queue.
    /// It then calls the `handle_message` function to process the message.
    ///
    /// @return Result<(), HareError>
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with the RabbitMQ connection or script execution.
    async fn rabbitmq_loop(&self) -> Result<(), HareError> {
        log::info!("Connecting to {}", self.rabbitmq_url);

        let connection = lapin::Connection::connect(&self.rabbitmq_url, lapin::ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;

        let mut consumer = channel.basic_consume(&self.queue_name, "hare_consumer", BasicConsumeOptions::default(), FieldTable::default()).await?;

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    self.handle_delivery(&delivery).await?;
                    delivery.ack(BasicAckOptions::default()).await?;
                },
                Err(error) => {
                    return Err(HareError::AmqpConnectionError(error));
                }
            }
        }
        Ok(())
    }

    /// Handles a delivery from the AMQP queue
    ///
    /// This function takes a delivery from the AMQP queue and handles it.
    /// It extracts the headers from the delivery and passes them to the `handle_message` function.
    ///
    /// # Arguments
    ///
    /// * `delivery` - The delivery to handle
    async fn handle_delivery(&self, delivery: &Delivery) -> Result<(), HareError> {

        // convert headers to map
        let mut header_map: HashMap<String, String> = HashMap::new();
        let headers = delivery.properties.headers().as_ref();
        match headers {
            Some(headers) => {
                for (k,v) in headers {
                    let key = k.as_str();
                    if let Some(str) = amqputils::get_string_value(v) {
                        header_map.insert(key.to_string(), str);
                    }
                }
            },
            None => {
                log::info!("No headers found");
            }
        }

        self.handle_message(header_map).await;

        Ok(())
    }

    async fn handle_message(&self, headers: HashMap<String, String>) -> Result<(), HareError> {

        if let Some(value) = headers.get("type") {
            // check if value is a alphanumeric string
            if value.chars().all(|c| c.is_alphanumeric()) {
                log::info!("Message type: {}", value);

                // make the script path
                let script_path = format!("{}/{}", self.script_root, value);

                // check if script at script_path exists
                let path = Path::new(&script_path);
                if path.exists() {
                    log::info!("Script found at {}", script_path);

                    // run the script
                    let mut environment: HashMap<String, String> = HashMap::new();

                    // copy headers into environment
                    for (k,v) in headers {
                        environment.insert(format!("HARE_VAR_{}", k.to_ascii_uppercase()), v);
                    }

                    let output = std::process::Command::new(script_path)
                        .envs(environment)
                        .output()
                        .expect("failed to execute script");
                    log::info!("Script output: {}", String::from_utf8_lossy(&output.stdout));
                } else {
                    log::info!("Script not found at {}", script_path);
                }
            } else {
                log::info!("message type {} not alphanumeric", value)
            }
        } else {
            log::info!("No type found in headers");
        }

        Ok(())
    }
}
