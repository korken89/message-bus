use async_bus::make_message_bus;

make_message_bus!(
    bus::Toplevel => {
        Topic1 [10] => u8,
        // Topic2 => u16,
        // Topic3 => u32,
        t1::SubTopic4 => {
            Topic5 [20] => u8,
            // Topic6 => u16,
            // Topic7 => u32,
        },
        t2::SubTopic8 => {
            Topic9 [30] => u8,
            // Topic10 => u16,
            // Topic11 => u32,
            t3::SubTopic12 => {
                Topic13 [40] => u8,
                // Topic14 => u16,
                // Topic15 => u32,
            },
        },
    },
);

#[tokio::main]
async fn main() {
    // ..
}
