mod serial_utils;

fn main() {
    let mut v = vec![0x10, 0x07];
    let add = vec![0x00, 0x00];

    v.splice(1..1, add);

    println!("{:02X?}", v);
}
