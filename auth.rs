import base64::{to_base64, from_base64};

export password;
export hasher;
export pbkdf2_sha1;
export default_pbkdf2_sha1;

fn password(hasher: hasher, password: @str) -> str {
    hasher.encode(password, @hasher.salt())
}

iface hasher {
    fn algorithm() -> str;
    fn salt() -> [u8];
    fn encode(pass: @str, salt: @[u8]) -> str;
    fn verify(pass: @str, encoded: @str) -> bool;
}

type pbkdf2_sha1 = {
    iterations: uint,
    keylen: uint
};

fn pbkdf2_sha1(iterations: uint, keylen: uint) -> hasher {
    { iterations: iterations, keylen: keylen } as hasher
}

fn default_pbkdf2_sha1() -> hasher {
    pbkdf2_sha1(10000u, 20u)
}

impl of hasher for pbkdf2_sha1 {
    fn algorithm() -> str { "pbkdf2_sha1" }

    fn salt() -> [u8] {
        crypto::rand::rand_bytes(self.keylen)
    }

    fn encode(pass: @str, salt: @[u8]) -> str {
        self.encode_iterations(*pass, *salt, self.iterations)
    }

    fn encode_iterations(pass: str, salt: [u8], iterations: uint) -> str {
        let hash = crypto::pkcs5::pbkdf2_hmac_sha1(pass, salt,
                                                   iterations,
                                                   self.keylen);

        let salt = salt.to_base64();
        let hash = hash.to_base64();

        #fmt("%s$%u$%s$%s", self.algorithm(), self.iterations, salt, hash)
    }

    fn verify(pass: @str, encoded: @str) -> bool {
        let parts = str::splitn_char(*encoded, '$', 3u);
        assert self.algorithm() == parts[0u];

        let iterations = uint::from_str(parts[1u]).get();
        let salt = parts[2u].from_base64();

        let encoded_2 = self.encode_iterations(*pass, salt, iterations);

        constant_time_compare_str(*encoded, encoded_2)
    }
}

fn constant_time_compare_vec(v1: [u8], v2: [u8]) -> bool {
    let len = v1.len();
    if len != v2.len() { ret false; }

    let mut i = 0u;
    let mut result = 0_u8;
    while i < len {
        result = result | (v1[i] ^ v2[i]);
        i += 1u;
    }

    result == 0_u8
}

fn constant_time_compare_str(s1: str, s2: str) -> bool {
    str::as_bytes(s1) { |s1_buf|
        str::as_bytes(s2) { |s2_buf|
            constant_time_compare_vec(s1_buf, s2_buf)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let hasher = pbkdf2_sha1(4096u, 20u);
        let encoded = hasher.encode("password", str::bytes("salt"));

        assert encoded ==
            "pbkdf2_sha1$4096$c2FsdA==$SwB5AbdlSJq+rUnZJvch0GWkKcE=";

        assert hasher.verify("password", encoded);
    }
}
