//! High-level APIs for doing operations over [`KernelConnection`] objects.

use super::{
    wire_protocol::{KernelInfoReply, KernelInfoRequest, KernelMessage, KernelMessageType, Reply},
    KernelConnection,
};
use crate::Error;

/// Get information through the KernelInfo command.
pub async fn kernel_info(conn: &KernelConnection) -> Result<KernelInfoReply, Error> {
    let mut req = conn
        .call_shell(KernelMessage::new(
            KernelMessageType::KernelInfoRequest,
            KernelInfoRequest {},
        ))
        .await?;
    let msg = req.get_reply::<KernelInfoReply>().await?;
    match msg.content {
        Reply::Ok(info) => Ok(info),
        Reply::Error(_) | Reply::Abort => Err(Error::KernelDisconnect),
    }
}
