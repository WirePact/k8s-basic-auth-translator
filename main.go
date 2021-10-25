package main

import (
	"flag"
	"fmt"
	"net"

	"github.com/sirupsen/logrus"
	"google.golang.org/grpc"
	"wirepact.ch/k8s-basic-auth-translator/certificates"
	"wirepact.ch/k8s-basic-auth-translator/translator"

	auth "github.com/envoyproxy/go-control-plane/envoy/service/auth/v3"
)

var (
	ingressPort = flag.Int("ingressPort", 50051, "The ingressPort that the server starts listening")
	egressPort  = flag.Int("egressPort", 50052, "The egressPort that the server starts listening")
)

func main() {
	flag.Parse()

	logrus.SetLevel(logrus.TraceLevel)

	listen, err := net.Listen("tcp", fmt.Sprintf(":%v", *ingressPort))
	if err != nil {
		logrus.Fatalf("Failed to listen to ingressPort: %v", *ingressPort)
	}

	certificates.EnsureSigningCertificate()

	var opts []grpc.ServerOption

	grpcServer := grpc.NewServer(opts...)
	auth.RegisterAuthorizationServer(grpcServer, &translator.IngressServer{})

	logrus.Infof("Starting server on address :%v", *ingressPort)
	err = grpcServer.Serve(listen)
	if err != nil {
		logrus.Fatalf("Server could not start: %v", err)
		return
	}
}
