pub fn demo_workcell() -> Vec<u8> {
    return include_str!("../../assets/demo_workcells/demo.workcell.json")
        .as_bytes()
        .to_vec();
}
