use data_flow_rs::configuration::get_configuration;
use dotenv::dotenv;
use ethers::providers::{Http, Middleware, Provider};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    dotenv().ok();
    let configuration = get_configuration().expect("Failed to get configuration.");

    let mut from_block;

    {
        let provider = Provider::<Http>::try_from(configuration.application.node_url_http.clone())
            .expect("Couldn't instantiate http provider");

        from_block = provider
            .get_block_number()
            .await
            .unwrap_or_else(|err| panic!("Couldn't get latest block. Error: {err}"))
            .as_u64();
    }

    from_block -= 10;

    data_flow_rs::startup::run(
        configuration.application.node_url_ws.to_owned(),
        configuration.application.node_url_http.to_owned(),
        configuration.application.block_buffer_size,
        from_block,
    )
    .await?;

    Ok(())
}
