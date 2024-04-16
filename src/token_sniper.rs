pub struct TokenSniper {
    // token: Token
}

impl TokenSniper {
    pub fn new() -> Self {
        Self {
            // token: Token{}
        }
    }

    // pub fn snipe(&self, mut new_token_receiver: Receiver<NewTokenEvent>) {
    //     tokio::spawn(async move {
    //         loop {
    //             while let Ok(token_event) = new_token_receiver.recv().await {
    //                 let new_token = match token_event {
    //                     NewTokenEvent::NewToken{ token} => token,
    //                 };

    //                 info!("New token event: {:?}", new_token);
    //             };
    //         }
    //     });
    // }
}