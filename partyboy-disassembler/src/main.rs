mod opcode;
mod tokenizer;

fn main() {
    let bin = std::fs::read("bin/_cgb_boot.bin").unwrap();

    let output = tokenizer::parse(&bin).unwrap();
    println!("{output:#?}");
}
