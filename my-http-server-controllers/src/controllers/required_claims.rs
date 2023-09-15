use my_http_server_core::RequestClaim;

#[derive(Debug, Clone)]
pub struct RequiredClaims {
    required_claims: Vec<String>,
}

impl RequiredClaims {
    pub fn no_claims() -> Self {
        Self {
            required_claims: vec![],
        }
    }

    pub fn from_vec(claims: Vec<String>) -> Self {
        Self {
            required_claims: claims,
        }
    }

    pub fn from_slice_of_str(claims: &[&str]) -> Self {
        let mut required_claims = Vec::with_capacity(claims.len());

        for claim in claims {
            required_claims.push(claim.to_string());
        }

        Self { required_claims }
    }

    pub fn authorized_by_claims(
        &self,
        request_ip: &str,
        request_claims: Option<Vec<RequestClaim>>,
    ) -> Option<String> {
        // No Claims means - we are authorized
        if self.required_claims.len() == 0 {
            return None;
        }

        if let Some(request_claims) = request_claims {
            for required_claim in &self.required_claims {
                let request_claim = request_claims.iter().find(|c| c.id == required_claim);

                let Some(request_claim) = request_claim else {
                    return Some(required_claim.to_owned());
                };

                if !request_claim.is_ip_allowed(request_ip) {
                    return Some(request_claim.id.to_string());
                }
            }

            return None;
        }

        let first_claim = &self.required_claims[0];
        Some(first_claim.to_string())
    }
}
