use tokenizer::parse;

mod opcode;
mod tokenizer;

fn main() {
    let bin = [0x00, 0x28, 0b1000_0010, 0x00, 0x18, 0b0000_1000];

    let output = parse(&bin).unwrap();
    println!("{output:#?}");
}
