package translator

import (
	"context"

	auth "github.com/envoyproxy/go-control-plane/envoy/service/auth/v3"
	"github.com/gogo/googleapis/google/rpc"
	"github.com/sirupsen/logrus"
	rpcstatus "google.golang.org/genproto/googleapis/rpc/status"
)

type IngressServer struct{}

func (a *IngressServer) Check(ctx context.Context, req *auth.CheckRequest) (*auth.CheckResponse, error) {
	// Basically, the check runs for every incoming request. If the request contains the
	// specific X-WirePact-Identity header, the header value is processed.
	// If not, then the request is just forwarded and therefore allowed to the target system.
	logger := logrus.
		WithContext(ctx).
		WithFields(logrus.Fields{
			"request_id": req.Attributes.Request.Http.Id,
			"host":       req.Attributes.Request.Http.Host,
			"path":       req.Attributes.Request.Http.Path,
			"method":     req.Attributes.Request.Http.Method,
		})
	logger.Traceln("Checking ingress request")

	_, ok := req.Attributes.Request.Http.Headers[wirepactIdentityHeader]
	if !ok {
		logger.Debugln("The request has no WirePact identity header. Skipping.")
		return &auth.CheckResponse{
			Status: &rpcstatus.Status{
				Code: int32(rpc.OK),
			},
			HttpResponse: &auth.CheckResponse_OkResponse{
				OkResponse: &auth.OkHttpResponse{},
			},
		}, nil
	}

	// TODO
	return &auth.CheckResponse{
		Status: &rpcstatus.Status{
			Code: int32(rpc.OK),
		},
		HttpResponse: &auth.CheckResponse_OkResponse{
			OkResponse: &auth.OkHttpResponse{},
		},
	}, nil
}
