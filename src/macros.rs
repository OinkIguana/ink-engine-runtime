macro_rules! id_eq {
    ($t:ty) => {
        impl std::cmp::PartialEq for $t {
            fn eq(&self, other: &Self) -> bool { self.id == other.id }
        }

        impl std::cmp::Eq for $t {}
    }
}

macro_rules! id_hash {
    ($t:ty) => {
        impl std::hash::Hash for $t {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state) }
        }
    }
}
