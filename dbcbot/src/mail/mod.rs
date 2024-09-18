pub mod model;
use model::Mail;

use crate::{database::PgDatabase, BotError};

pub trait MailDatabase {
    type Error;
    async fn store(&self, mail: Mail) -> Result<(), Self::Error>;
    async fn get(&self) -> Result<Option<Mail>, Self::Error>;
    async fn get_all_unread(&self, user: String) -> Result<Vec<Mail>, Self::Error>;
}

// impl MailDatabase for PgDatabase{
//     type Error = BotError;
//     async fn get(&self) -> Result<Option<Mail>, Self::Error>{

//         Ok(None)
//     }

//     async fn get_all(&self, user: String) -> Result<Vec<Mail>, Self::Error> {
//         Ok(vec![])
//     }
//     async fn store(&self, mail: Mail) -> Result<(), Self::Error>{

//         Ok(())
//     }
// }
