package translator

import (
	"context"
	"encoding/base64"
	"strings"

	core "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	auth "github.com/envoyproxy/go-control-plane/envoy/service/auth/v3"
	types "github.com/envoyproxy/go-control-plane/envoy/type/v3"
	"github.com/gogo/googleapis/google/rpc"
	"github.com/sirupsen/logrus"
	rpcstatus "google.golang.org/genproto/googleapis/rpc/status"
	"wirepact.ch/k8s-basic-auth-translator/user_repository"
	"wirepact.ch/k8s-basic-auth-translator/wirepact_jwt"
)

type EgressServer struct{}

func (a *EgressServer) Check(ctx context.Context, req *auth.CheckRequest) (*auth.CheckResponse, error) {
	// Basically, the check runs for every incoming request. If the request contains the
	// specific X-WirePact-Identity header, the header value is processed.
	// If not, then the request is just forwarded and therefore allowed to the target system.
	logger := logrus.
		WithContext(ctx).
		WithFields(logrus.Fields{
			"type":       "egress",
			"request_id": req.Attributes.Request.Http.Id,
			"host":       req.Attributes.Request.Http.Host,
			"path":       req.Attributes.Request.Http.Path,
			"method":     req.Attributes.Request.Http.Method,
		})
	logger.Traceln("Checking egress request.")

	// For the egress (outgoing) communication, the basic auth translator
	// checks if there is an authorization header. If so, the header is
	// consumed (removed) from the original request and a converted and signed
	// JWT token is attached to the wirepact identity header.

	header, ok := req.Attributes.Request.Http.Headers[authorizationHeader]
	if !ok {
		logger.Debugln("The request has no authorization header. Skipping.")
		return &auth.CheckResponse{
			Status: &rpcstatus.Status{
				Code: int32(rpc.OK),
			},
			HttpResponse: &auth.CheckResponse_OkResponse{
				OkResponse: &auth.OkHttpResponse{},
			},
		}, nil
	} else if !strings.Contains(header, "Basic") {
		logger.Debugln("The request is not Basic Auth authorized. Skipping.")
		return &auth.CheckResponse{
			Status: &rpcstatus.Status{
				Code: int32(rpc.OK),
			},
			HttpResponse: &auth.CheckResponse_OkResponse{
				OkResponse: &auth.OkHttpResponse{},
			},
		}, nil
	}

	logger.Debugln("The request contains a Basic Auth signature. Convert to WirePact JWT.")

	payload, _ := base64.StdEncoding.DecodeString(strings.ReplaceAll(header, "Basic ", ""))
	authPair := strings.SplitN(string(payload), ":", 2)

	if len(authPair) != 2 {
		logger.Warnf("The Basic Auth data was corruped. Received '%v'. Not Authorizing Request.", string(payload))
		return &auth.CheckResponse{
			Status: &rpcstatus.Status{
				Code: int32(rpc.UNAUTHENTICATED),
			},
			HttpResponse: &auth.CheckResponse_DeniedResponse{
				DeniedResponse: &auth.DeniedHttpResponse{
					Body:   "The Basic Auth data was corrupted.",
					Status: &types.HttpStatus{Code: types.StatusCode_Unauthorized},
				},
			},
		}, nil
	}

	username := authPair[0]
	password := authPair[1]
	repository := user_repository.GetUserRepository()

	userID := repository.LookupUserID(username, password)
	if userID == nil {
		logger.Infof("No userID found for user '%v'.", username)
		return &auth.CheckResponse{
			Status: &rpcstatus.Status{
				Code: int32(rpc.PERMISSION_DENIED),
			},
			HttpResponse: &auth.CheckResponse_DeniedResponse{
				DeniedResponse: &auth.DeniedHttpResponse{
					Body:   "No userID found for user.",
					Status: &types.HttpStatus{Code: types.StatusCode_Forbidden},
				},
			},
		}, nil
	}

	logger.Infof("UserID found for user '%v'. Return OK with signed JWT.", username)
	return &auth.CheckResponse{
		Status: &rpcstatus.Status{
			Code: int32(rpc.OK),
		},
		HttpResponse: &auth.CheckResponse_OkResponse{
			OkResponse: &auth.OkHttpResponse{
				Headers: []*core.HeaderValueOption{
					{
						Header: &core.HeaderValue{
							Key:   wirepactIdentityHeader,
							Value: wirepact_jwt.CreateSignedJWTForUser(*userID),
						},
					},
				},
				HeadersToRemove: []string{authorizationHeader},
			},
		},
	}, nil
}
