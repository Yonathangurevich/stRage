use clap::Parser;

#[derive(Parser)]
#[command(name = "stRage")]
#[command(version = "1.0")]
#[command(about = "AI review", long_about = None)]
struct Cli {
    #[arg(short, long)]
    review: bool
}

fn main() {
    let args = Cli::parse();
    if args.review {
       println!("the review is ON");
    }
}
