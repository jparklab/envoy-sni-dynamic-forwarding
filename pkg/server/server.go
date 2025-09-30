package server

import (
	"context"
	"log"

	authz "github.com/envoyproxy/go-control-plane/envoy/service/auth/v3"
	typepb "github.com/envoyproxy/go-control-plane/envoy/type/v3"
	status "google.golang.org/genproto/googleapis/rpc/status"
	"google.golang.org/grpc/codes"
	"google.golang.org/protobuf/types/known/structpb"
)

type authorizationServer struct {
}

var _ authz.AuthorizationServer = &authorizationServer{}

func NewAuthorizationServer() *authorizationServer {
	log.Println("Create a new authorization server")
	return &authorizationServer{}
}

func (s *authorizationServer) Check(ctx context.Context, req *authz.CheckRequest) (*authz.CheckResponse, error) {
	log.Printf("Received a check request: %v\n", req)

	sni := req.GetAttributes().GetTlsSession().GetSni()

	if sni == "" {
		log.Println("No SNI provided")
		return &authz.CheckResponse{
			Status: &status.Status{
				Code: int32(codes.PermissionDenied),
			},
			HttpResponse: &authz.CheckResponse_DeniedResponse{
				DeniedResponse: &authz.DeniedHttpResponse{
					Status: &typepb.HttpStatus{Code: 403},
				},
			},
		}, nil
	}

	metadata, err := structpb.NewStruct(
		map[string]interface{}{
			"envoy.upstream.dynamic_host": "127.0.0.1",
			"envoy.upstream.dynamic_port": "19443",
		},
	)
	if err != nil {
		log.Printf("Failed to create struct: %v\n", err)
		return &authz.CheckResponse{
			Status: &status.Status{
				Code: int32(codes.Internal),
			},
		}, nil
	}

	log.Printf("Routing to 127.0.0.1:19443 for %s", sni)
	return &authz.CheckResponse{
		Status: &status.Status{
			Code: int32(codes.OK),
		},
		DynamicMetadata: metadata,
		HttpResponse: &authz.CheckResponse_OkResponse{
			OkResponse: &authz.OkHttpResponse{},
		},
	}, nil
}
