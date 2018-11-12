// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use encoding::protobuf::sandbox::TimestampTx;

use rand::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

use blockchain::{ExecutionResult, Service, Transaction, TransactionContext, TransactionSet};
use crypto::{gen_keypair, Hash, PublicKey, SecretKey, HASH_SIZE};
use encoding::Error as MessageError;
use messages::{BinaryForm, Message, RawTransaction, ServiceTransaction, Signed};
use protobuf::Message as PbMessage;
use storage::Snapshot;

pub const TIMESTAMPING_SERVICE: u16 = 129;
pub const DATA_SIZE: usize = 64;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TimestampingTransactions {
    TimestampTx(TimestampTx),
}

impl Transaction for TimestampTx {
    fn execute(&self, _: TransactionContext) -> ExecutionResult {
        Ok(())
    }
}

impl TransactionSet for TimestampingTransactions {
    fn tx_from_raw(raw: RawTransaction) -> Result<Self, MessageError> {
        let (id, vec) = raw.service_transaction().into_raw_parts();
        match id {
            0 => {
                let mut ts_tx = TimestampTx::new();
                ts_tx.merge_from_bytes(&vec).unwrap();
                Ok(TimestampingTransactions::TimestampTx(ts_tx))
            }
            num => Err(MessageError::Basic(
                format!(
                    "Tag {} not found for enum {}.",
                    num, "TimestampingTransactions"
                ).into(),
            )),
        }
    }
}

impl Into<ServiceTransaction> for TimestampingTransactions {
    fn into(self) -> ServiceTransaction {
        let (id, vec) = match self {
            TimestampingTransactions::TimestampTx(ref tx) => (0, tx.write_to_bytes().unwrap()),
        };
        ServiceTransaction::from_raw_unchecked(id, vec)
    }
}

impl Into<TimestampingTransactions> for TimestampTx {
    fn into(self) -> TimestampingTransactions {
        TimestampingTransactions::TimestampTx(self)
    }
}

impl Into<ServiceTransaction> for TimestampTx {
    fn into(self) -> ServiceTransaction {
        let set: TimestampingTransactions = self.into();
        set.into()
    }
}

impl Into<Box<dyn Transaction>> for TimestampingTransactions {
    fn into(self) -> Box<dyn Transaction> {
        match self {
            TimestampingTransactions::TimestampTx(tx) => Box::new(tx),
        }
    }
}

#[derive(Default)]
pub struct TimestampingService {}

pub struct TimestampingTxGenerator {
    rand: XorShiftRng,
    data_size: usize,
    public_key: PublicKey,
    secret_key: SecretKey,
}

impl TimestampingTxGenerator {
    pub fn new(data_size: usize) -> TimestampingTxGenerator {
        let keypair = gen_keypair();
        TimestampingTxGenerator::with_keypair(data_size, keypair)
    }

    pub fn with_keypair(
        data_size: usize,
        keypair: (PublicKey, SecretKey),
    ) -> TimestampingTxGenerator {
        let rand = XorShiftRng::from_seed([9; 16]);

        TimestampingTxGenerator {
            rand,
            data_size,
            public_key: keypair.0,
            secret_key: keypair.1,
        }
    }
}

impl Iterator for TimestampingTxGenerator {
    type Item = Signed<RawTransaction>;

    fn next(&mut self) -> Option<Signed<RawTransaction>> {
        let mut data = vec![0; self.data_size];
        self.rand.fill_bytes(&mut data);
        let mut buf = TimestampTx::new();
        buf.set_data(data);
        Some(Message::sign_transaction(
            buf,
            TIMESTAMPING_SERVICE,
            self.public_key,
            &self.secret_key,
        ))
    }
}

impl TimestampingService {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Service for TimestampingService {
    fn service_name(&self) -> &str {
        "sandbox_timestamping"
    }

    fn service_id(&self) -> u16 {
        TIMESTAMPING_SERVICE
    }

    fn state_hash(&self, _: &dyn Snapshot) -> Vec<Hash> {
        vec![Hash::new([127; HASH_SIZE]), Hash::new([128; HASH_SIZE])]
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<dyn Transaction>, MessageError> {
        let tx = TimestampingTransactions::tx_from_raw(raw)?;
        Ok(tx.into())
    }
}
