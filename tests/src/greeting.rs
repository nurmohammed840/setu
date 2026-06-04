use setu::{Input, Output};

/// The request message containing the user's name.
#[derive(Input)]
pub struct HelloRequest {
    #[key = 1]
    pub name: String,
}

/// The response message containing the greetings.
#[derive(Output)]
pub struct HelloReply {
    #[key = 1]
    pub message: String,
}

pub async fn say_hello(input: HelloRequest) -> HelloReply {
    HelloReply {
        message: format!("Hello {}!", input.name),
    }
}
