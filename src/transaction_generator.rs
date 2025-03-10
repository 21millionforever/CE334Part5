use serde::{Serialize,Deserialize};
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use crate::address::H160;
use crate::crypto::hash::{H256, Hashable};

use crate::network::server::Handle as ServerHandle;
use crate::transaction::{RawTransaction, SignedTransaction};
use std::thread;
use std::time;
use std::sync::{Arc, Mutex};
use crate::mempool::Mempool;
use crate::network::message::Message;
use crate::blockchain::{Blockchain};

pub struct TransactionGenerator {
    server: ServerHandle,
    mempool: Arc<Mutex<Mempool>>,
    blockchain: Arc<Mutex<Blockchain>>,
    controlled_keypair: Ed25519KeyPair,
}

impl TransactionGenerator {
    pub fn new(
        server: &ServerHandle,
        mempool: &Arc<Mutex<Mempool>>,
        blockchain: &Arc<Mutex<Blockchain>>,
        controlled_keypair: Ed25519KeyPair
    ) -> TransactionGenerator {
        TransactionGenerator {
            server: server.clone(),
            mempool: Arc::clone(mempool),
            blockchain: Arc::clone(blockchain),
            controlled_keypair,
        }
    }

    pub fn start(self) {
        thread::spawn(move || {
            self.generation_loop();
            log::warn!("Transaction Generator exited");
        });
    }

    /// Generate random transactions and send them to the server
    fn generation_loop(&self) {
        const INTERVAL_MILLISECONDS: u64 = 3000; // how quickly to generate transactions

        loop {
            // sleep for some time:
            let interval = time::Duration::from_millis(INTERVAL_MILLISECONDS);
            thread::sleep(interval);

            // 1. generate some random transactions:
            let raw_transaction = RawTransaction {
                from_addr: H160::from_pubkey(self.controlled_keypair.public_key().as_ref()),
                to_addr: H160::from_pubkey(self.controlled_keypair.public_key().as_ref()), // for example, send to self
                value: 10,
                nonce: 0, // update as needed
            };
            let signed_transaction = SignedTransaction::from_raw(raw_transaction, &self.controlled_keypair);
            // 2. add these transactions to the mempool:
            let mut mempool = self.mempool.lock().unwrap();
            mempool.insert(signed_transaction.clone());

            // 3. broadcast them using `self.server.broadcast(Message::NewTransactionHashes(...))`:
            self.server.broadcast(Message::NewTransactionHashes(vec![signed_transaction.raw.hash()]));
        }
    }
}
