use clap::{Args, Parser, Subcommand};
use mycalc::{add, div, mul, sub};

#[derive(Parser, Debug)]
#[command(
    name = "mycalc",
    version,
    about = "Simple CLI calculator with clap (add, sub, mul, div)",
    arg_required_else_help = true,
    propagate_version = true,
    disable_help_subcommand = true
)]
struct Cli {
    /// Presition of decimal places when printing (default 2)
    #[arg(global = true, short, long, default_value_t = 2)]
    precision: usize,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add all numbers: mycalc add 1 2 3
    Add(OpArgs),
    /// Subtract (left-associative): mycalc sub 10 3 2  => (10 - 3 - 2)
    Sub(OpArgs),
    /// Multiply all numbers: mycalc mul 2 3 4
    Mul(OpArgs),
    /// Divide is (left-associative): mycalc div 20 2 5 => (20 / 2 / 5)
    Div(OpArgs),
}

#[derive(Args, Debug)]
struct OpArgs {
    /// Numbers to operate (at least 2)
    #[arg(value_name = "NUM", num_args = 2..)]
    nums: Vec<f64>,
}

fn main() {
    let cli = Cli::parse();
    let precision = cli.precision;

    match cli.command {
        Commands::Add(args) => {
            let result = add(&args.nums);
            println!("{:.*}", precision, result);
        }
        Commands::Sub(args) => {
            let result = sub(&args.nums);
            println!("{:.*}", precision, result);
        }
        Commands::Mul(args) => {
            let result = mul(&args.nums);
            println!("{:.*}", precision, result);
        }
        Commands::Div(args) => match div(&args.nums) {
            Ok(result) => println!("{:.*}", precision, result),
            Err(msg) => {
                eprintln!("{}", msg);
                std::process::exit(1);
            }
        },
    }
}
