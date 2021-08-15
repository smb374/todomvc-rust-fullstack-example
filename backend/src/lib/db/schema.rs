table! {
    task (id) {
        id -> Uuid,
        content -> Text,
        completed -> Bool,
        editing -> Bool,
    }
}
