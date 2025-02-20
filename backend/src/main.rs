
fn main() {
    let ports: Vec<_> = match serialport::available_ports() {
        Ok(val) => val,
        Err(_) => panic!("welp, no ports, thats weird")
    };
    
    
    for i in ports {
        println!("{}", i.port_name);
    }
}
