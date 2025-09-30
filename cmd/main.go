package main

import (
	"context"
	"fmt"
	"log"
	"net"
	"net/rpc"
	"time"

	"github.twosigma.com/jparklab/ext-authz/pkg/server"
	"google.golang.org/grpc"
	"google.golang.org/grpc/keepalive"
	"google.golang.org/grpc/reflection"

	authz "github.com/envoyproxy/go-control-plane/envoy/service/auth/v3"
	"github.com/soheilhy/cmux"
	"github.com/spf13/pflag"
)

var (
	port = pflag.Int("port", 9000, "The server port")
)

func main() {
	pflag.Parse()

	lis, err := net.Listen(
		"tcp",
		fmt.Sprintf(":%d", *port),
	)
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}

	grpcServer := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (resp any, err error) {
				log.Printf("request: %v", req)
				return handler(ctx, req)
			},
		),
		// Configure keepalive parameters to handle HTTP/2 properly
		grpc.KeepaliveParams(keepalive.ServerParameters{
			MaxConnectionIdle:     15 * time.Second, // If a client is idle for 15 seconds, send a GOAWAY
			MaxConnectionAge:      30 * time.Second, // If any connection is alive for more than 30 seconds, send a GOAWAY
			MaxConnectionAgeGrace: 5 * time.Second,  // Allow 5 seconds for pending RPCs to complete before forcibly closing connections
			Time:                  5 * time.Second,  // Ping the client if it is idle for 5 seconds to ensure the connection is still active
			Timeout:               1 * time.Second,  // Wait 1 second for the ping ack before assuming the connection is dead
		}),
		// Configure keepalive enforcement policy
		grpc.KeepaliveEnforcementPolicy(keepalive.EnforcementPolicy{
			MinTime:             5 * time.Second, // If a client pings more than once every 5 seconds, terminate the connection
			PermitWithoutStream: true,            // Allow pings even when there are no active streams
		}),
		// Set max receive message size (4MB)
		grpc.MaxRecvMsgSize(4*1024*1024),
		// Set max send message size (4MB)
		grpc.MaxSendMsgSize(4*1024*1024),
	)

	m := cmux.New(lis)
	grpcLis := m.MatchWithWriters(cmux.HTTP2MatchHeaderFieldSendSettings("content-type", "application/grpc"))
	trpcL := m.Match(cmux.Any()) // Any means anything that is not yet matched.

	extAuthzServer := server.NewAuthorizationServer()
	authz.RegisterAuthorizationServer(grpcServer, extAuthzServer)

	trpcS := rpc.NewServer()

	// Register reflection service on gRPC server
	reflection.Register(grpcServer)

	log.Printf("server listening at %v", lis.Addr())
	log.Printf("gRPC server configured with HTTP/2 keepalive settings")

	if true {
		// run muxed listeners
		go grpcServer.Serve(grpcLis)
		go trpcS.Accept(trpcL)

		// start serving
		m.Serve()
	} else {
		grpcServer.Serve(lis)
	}
}
