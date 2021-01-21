use kataru::pack;

fn main() {
    // Pack all story files into MessagePack binaries to be embedded in build.
    pack("./kataru", "./target").unwrap();
}
