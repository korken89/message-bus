use message_bus_macros::make_message_bus;

make_message_bus!(
    bus::Toplevel => {
        SomeData => u32, // Err
        SomeData => i32, // Err
        SomeData2 => u32, // Err
        SomeData2 => i32, // Err
        sub::Topic => {
            SomeData => u32, // Ok
            SomeData2 => u32, // Ok
            SomeData3 => u32, // Err
            SomeData3 => u32, // Err
            sub::Topic2 => {
                SomeData => u32, // Ok
                SomeData2 => u32, // Ok
                SomeData3 => u32, // Ok
                SomeData4 => u32, // Err
                SomeData4 => u32, // Err
            },
        },
    },
);

fn main() {}
