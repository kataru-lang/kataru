use kataru::pack;

fn main() {
    // Pack all story files into MessagePack binaries to be embedded in build.
    pack(
        "C:\\Users\\Joshi\\Dev\\Unity\\JongelaMirrors\\Assets\\Kataru",
        "./target",
    )
    .unwrap();
}
