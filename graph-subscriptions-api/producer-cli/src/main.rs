use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use datasource::{GatewaySubscriptionQueryResult, StatusCode};
use prost::Message;
use rdkafka::{
    producer::{BaseRecord, DefaultProducerContext, ThreadedProducer},
    ClientConfig,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Opt {
    #[arg(
        long,
        help = "the kafka broker url",
        default_value = "plaintext://localhost:9092"
    )]
    broker: String,
    #[arg(
        long,
        help = "the kafka topic to send the messages on",
        default_value = "gateway_subscription_query_results"
    )]
    topic_id: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Produce {
        #[arg(long)]
        query_id: String,
        #[arg(
            long,
            help = "Success = 0, InternalError = 1, UserError = 2, NotFound = 3",
            default_value = "0"
        )]
        status_code: Option<i32>,
        #[arg(long)]
        status_message: String,
        #[arg(long)]
        response_time_ms: u32,
        #[arg(long, help = "if no value provided, will use ticket_signer")]
        ticket_user: Option<String>,
        #[arg(long)]
        ticket_signer: String,
        #[arg(long)]
        ticket_name: String,
        #[arg(long)]
        deployment: String,
        #[arg(long, help = "the chain the subgraph is indexing")]
        chain: Option<String>,
        #[arg(
            long,
            help = "the count of queries made by the user, on the deployment"
        )]
        query_count: Option<u32>,
        #[arg(long, help = "the budget chosen by the user")]
        query_budget: Option<f32>,
        #[arg(long, help = "the fees charged by the indexer")]
        indexer_fees: Option<f32>,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opt = Opt::parse();
    eprintln!("{:#?}", opt);

    let producer: &ThreadedProducer<DefaultProducerContext> = &ClientConfig::new()
        .set("bootstrap.servers", opt.broker)
        .set("message.timeout.ms", "5000")
        .create_with_context(DefaultProducerContext)
        .expect("could not connect to the kafka instance to build the producer");

    match opt.command {
        Commands::Produce {
            query_id,
            status_code,
            status_message,
            response_time_ms,
            ticket_user,
            ticket_signer,
            ticket_name,
            deployment,
            chain,
            query_count,
            query_budget,
            indexer_fees,
        } => {
            let message = GatewaySubscriptionQueryResult {
                query_id,
                status_code: status_code.unwrap_or(StatusCode::Success.into()),
                status_message,
                response_time_ms,
                ticket_user: ticket_user.unwrap_or(ticket_signer.to_string()),
                ticket_signer: ticket_signer,
                ticket_name: Some(ticket_name),
                deployment: Some(deployment),
                subgraph_chain: chain,
                query_count: Some(query_count.unwrap_or(0)),
                query_budget,
                indexer_fees,
            };
            // write the message to the producer as a protobuf message
            let message_data = message.encode_to_vec();
            let record = BaseRecord::<'_, (), [u8]>::to(&opt.topic_id).payload(&message_data);
            println!("sending message on topic {}", opt.topic_id);
            if let Err((kafka_producer_err, _)) = producer.send(record) {
                println!(
                    "failure sending message on topic: [{}]. error [{:?}]",
                    opt.topic_id, kafka_producer_err
                );
            } else {
                println!("message sent successfully on topic {}", opt.topic_id);
            }
        }
    }

    Ok(())
}
