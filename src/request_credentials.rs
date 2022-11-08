use rust_extensions::date_time::DateTimeAsMicroseconds;

pub trait RequestClaim {
    fn get_id(&self) -> &str;
    fn get_expires(&self) -> DateTimeAsMicroseconds;
    fn get_allowed_ips(&self) -> Option<&Vec<String>>;
}

pub trait RequestCredentials {
    fn get_id(&self) -> &str;
    fn get_claims<'s, TIterator: Iterator<Item = &'s dyn RequestClaim>>(
        &'s self,
    ) -> Option<TIterator>;
}
