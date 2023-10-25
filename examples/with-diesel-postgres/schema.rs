// @generated automatically by Diesel CLI.

diesel::table! {
    todos (uuid) {
        uuid -> Uuid,
        task -> Text,
        done -> Bool,
    }
}
