
fn print_usage() -> ! {
    println!("Usage");
    std::process::exit(1);
}

fn parse_arg<T: std::str::FromStr>(nth: usize) -> T {
    match std::env::args().nth(nth) {
        Some(arg) => match arg.parse::<T>() {
            Ok(targ) => targ,
            Err(_) => print_usage()
        },
        None => print_usage()
    }
}

fn main() {
    if std::env::args().count() != 3 {
        print_usage();
    }

    let width: usize = parse_arg(1);
    let height: usize = parse_arg(2);

    println!("width is {}", &width);
    println!("height is {}", &height);


}
