use sha2::{Digest, Sha256};
use sha3::Keccak256;

pub type MerkleTreeData = Vec<u8>;
pub type MerkleTreeHash = [u8; 32];
pub type MerkleTreeProof = Vec<MerkleTreeHash>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MerkleTreeRoot {
    pub hash: MerkleTreeHash,
}

pub struct MerkleTree {
    pub root: MerkleTreeRoot,
    pub proofs: Vec<MerkleTreeProof>,
}

impl MerkleTreeRoot {
    pub fn new(hash: MerkleTreeHash) -> Self {
        MerkleTreeRoot { hash }
    }

    pub fn verify(&self, data: &MerkleTreeData, proof: &MerkleTreeProof) -> bool {
        let mut hash = keccak256_array(data);
        for second_hash in proof {
            let s = serde_json::to_vec(&sort_hash_pair(&hash, second_hash)).unwrap();
            hash = keccak256_array(&s);
        }
        self.hash == hash
    }
}

pub fn keccak256_array(data: &[u8]) -> MerkleTreeHash {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

pub fn sort_hash_pair(
    first: &MerkleTreeHash,
    second: &MerkleTreeHash,
) -> (MerkleTreeHash, MerkleTreeHash) {
    if first < second {
        (first.clone(), second.clone())
    } else {
        (second.clone(), first.clone())
    }
}

impl MerkleTree {
    pub fn build(items: &Vec<MerkleTreeData>) -> Self {
        let items_len = items.len();

        let mut items = items.clone();

        let mut st_sum = 0_usize;
        let mut st = 1_usize;

        while st < items.len() {
            st_sum += st;

            st <<= 1;
        }

        while items.len() < st_sum + st {
            items.push(MerkleTreeData::new());
        }

        let mut nodes = vec![[0_u8; 32]; st_sum + st];

        for i in st_sum..st_sum + st {
            nodes[i] = keccak256_array(&items[i - st_sum]);
        }

        let mut i = st_sum.clone();

        while i > 0 {
            i -= 1;

            let s = serde_json::to_vec(&sort_hash_pair(&nodes[(i << 1) + 1], &nodes[(i + 1) << 1]))
                .unwrap();

            nodes[i] = keccak256_array(&s);
        }

        let get_proof = |index: usize| -> MerkleTreeProof {
            let mut result = MerkleTreeProof::new();

            let mut v = index + st_sum;

            while v > 0 {
                let w = if v % 2 == 0 { v - 1 } else { v + 1 };

                result.push(nodes[w]);

                v = (v - 1) >> 1;
            }

            result
        };

        let mut proofs: Vec<MerkleTreeProof> = Vec::new();

        for i in 0..items_len {
            proofs.push(get_proof(i))
        }

        MerkleTree {
            root: MerkleTreeRoot::new(nodes[0]),
            proofs,
        }
    }
}

pub fn string_to_crypto_hash(input: &str) -> MerkleTreeHash {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    let mut crypto_hash: MerkleTreeHash = [0u8; 32];
    crypto_hash.copy_from_slice(&result);

    crypto_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn correct_proofs() {
        let mut items = Vec::<MerkleTreeData>::new();

        for i in 0..8 {
            items.push(vec![i]);
        }

        let merkle_tree = MerkleTree::build(&items);

        assert_eq!(merkle_tree.proofs.len(), 8);

        for proof in merkle_tree.proofs {
            assert_eq!(proof.len(), 3);
        }
    }

    #[test]
    pub fn verify_correct_data() {
        let mut items = Vec::<MerkleTreeData>::new();

        for i in 0..4 {
            items.push(vec![i]);
        }

        let merkle_tree = MerkleTree::build(&items);
        println!("root - {:?}", merkle_tree.root);
        println!("proof - {:?} ---  {:?}", merkle_tree.proofs[0], items[0]);

        for i in 0..items.len() {
            assert!(merkle_tree.root.verify(&items[i], &merkle_tree.proofs[i]));
        }
    }

    #[test]
    pub fn verify_incorrect_data() {
        let mut items = Vec::<MerkleTreeData>::new();

        for i in 0..4 {
            items.push(vec![i]);
        }

        let merkle_tree = MerkleTree::build(&items);

        assert!(!merkle_tree.root.verify(&items[0], &merkle_tree.proofs[2]));
    }

    #[test]
    fn test_make_crypt_hash() {
        let s = "abcd".to_string();
        for x in 0..10 {
            let stx = format!("{}:{}", x, &s);
            let hash = string_to_crypto_hash(&stx);
            println!("hash - 0x{}", hex::encode(hash));
        }
    }
}

#[test]
fn testx() {
    let s = "abcd";
    let hash = keccak256_array(s.as_bytes());
    println!("h--{:?} ", hash);
}
