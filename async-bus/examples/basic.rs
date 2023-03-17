mod bus {
    use async_bus::make_message_bus;

    make_message_bus!(
        Topic1 => u8,
        Topic2 => u16,
        Topic3 => u32,
        t1::Topic4 => {
            Topic5 => u8,
            Topic6 => u16,
            Topic7 => u32,
        },
        t2::Topic8 => {
            Topic9 => u8,
            Topic10 => u16,
            Topic11 => u32,
            t3::Topic12 => {
                Topic13 => u8,
                Topic14 => u16,
                Topic15 => u32,
            },
        },
    );
}

#[tokio::main]
async fn main() {
    // ..
}
