use std::{io, net::SocketAddr};

use hickory_proto::{
    op::{Header, LowerQuery},
    rr::Record,
    serialize::binary::BinEncoder,
};
use hickory_server::{
    authority::MessageResponse,
    server::{ResponseHandler, ResponseInfo},
};
use tokio::sync::mpsc;

use crate::socket::SendDnsMessage;

#[derive(Clone)]
pub struct LandscapeResponse {
    dst: SocketAddr,
    sender: mpsc::Sender<SendDnsMessage>,
}

impl LandscapeResponse {
    pub fn new(dst: SocketAddr, sender: mpsc::Sender<SendDnsMessage>) -> Self {
        Self { dst, sender }
    }

    fn max_size_for_response<'a>(
        &self,
        response: &MessageResponse<
            '_,
            'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
        >,
    ) -> u16 {
        // Use EDNS, if available.
        if let Some(edns) = response.get_edns() {
            edns.max_payload()
        } else {
            // No EDNS, use the recommended max from RFC6891.
            hickory_proto::udp::MAX_RECEIVE_BUFFER_SIZE as u16
        }
    }
}

#[async_trait::async_trait]
impl ResponseHandler for LandscapeResponse {
    async fn send_response<'a>(
        &mut self,
        response: MessageResponse<
            '_,
            'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
        >,
    ) -> io::Result<ResponseInfo> {
        tracing::debug!(
            "response: {} response_code: {}",
            response.header().id(),
            response.header().response_code(),
        );
        let mut buffer = Vec::with_capacity(512);
        let encode_result = {
            let mut encoder = BinEncoder::new(&mut buffer);

            // Set an appropriate maximum on the encoder.
            let max_size = self.max_size_for_response(&response);
            tracing::trace!("setting response max size: {max_size}",);
            encoder.set_max_size(max_size);

            response.destructive_emit(&mut encoder)
        };

        let info = encode_result.map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("error encoding message: {e}"))
        })?;

        self.sender
            .send(SendDnsMessage { message: buffer, addr: self.dst })
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "unknown"))?;

        Ok(info)
    }
}
#[derive(Clone)]
pub(crate) struct ReportingResponseHandler<R: ResponseHandler> {
    pub request_header: Header,
    pub queries: Vec<LowerQuery>,
    pub protocol: hickory_proto::xfer::Protocol,
    pub src_addr: SocketAddr,
    pub handler: R,
}

#[async_trait::async_trait]
#[allow(clippy::uninlined_format_args)]
impl<R: ResponseHandler> ResponseHandler for ReportingResponseHandler<R> {
    async fn send_response<'a>(
        &mut self,
        response: hickory_server::authority::MessageResponse<
            '_,
            'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
            impl Iterator<Item = &'a Record> + Send + 'a,
        >,
    ) -> io::Result<ResponseInfo> {
        let response_info = self.handler.send_response(response).await?;

        let id = self.request_header.id();
        let rid = response_info.id();
        if id != rid {
            tracing::warn!("request id:{id} does not match response id:{rid}");
            debug_assert_eq!(id, rid, "request id and response id should match");
        }

        let rflags = response_info.flags();
        let answer_count = response_info.answer_count();
        let authority_count = response_info.name_server_count();
        let additional_count = response_info.additional_count();
        let response_code = response_info.response_code();

        tracing::info!(
            "request:{id} src:{proto}://{addr}#{port} {op} qflags:{qflags} response:{code:?} rr:{answers}/{authorities}/{additionals} rflags:{rflags}",
            id = rid,
            proto = self.protocol,
            addr = self.src_addr.ip(),
            port = self.src_addr.port(),
            op = self.request_header.op_code(),
            qflags = self.request_header.flags(),
            code = response_code,
            answers = answer_count,
            authorities = authority_count,
            additionals = additional_count,
            rflags = rflags
        );
        for query in self.queries.iter() {
            tracing::info!(
                "query:{query}:{qtype}:{class}",
                query = query.name(),
                qtype = query.query_type(),
                class = query.query_class()
            );
        }

        Ok(response_info)
    }
}
