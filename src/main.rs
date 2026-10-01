mod path;

use gateway_plugin_sdk::Manifest;
use gateway_plugin_sdk::client::{HttpCall, PluginBuilder, PluginSession, SessionConfig, SessionError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::from_author_slice(include_bytes!("../plugin.json"))?;
    let session = PluginSession::accept(
        tokio::io::stdin(),
        tokio::io::stdout(),
        SessionConfig::default(),
    )
    .await?;
    if session.handshake().plugin_id != manifest.plugin_id()?
        || session.handshake().contributes != manifest.contributes
        || !session.handshake().configuration.as_object().is_some_and(|value| value.is_empty())
    {
        return Err(SessionError::Handshake.into());
    }
    let plugin = PluginBuilder::from_manifest(manifest)?
        .middleware(|call: HttpCall| async move {
            let HttpCall { mut request, next, .. } = call;
            path::rewrite(&request.method, &mut request.uri);
            next.run(request).await
        })?
        .build()?;
    session.run(plugin).await?;
    Ok(())
}
