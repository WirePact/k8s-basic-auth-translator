package main

import (
	"flag"
	"fmt"
	"net"

	"github.com/sirupsen/logrus"
	"google.golang.org/grpc"
	"wirepact.ch/k8s-basic-auth-translator/pki"
	"wirepact.ch/k8s-basic-auth-translator/translator"
	"wirepact.ch/k8s-basic-auth-translator/user_repository"

	auth "github.com/envoyproxy/go-control-plane/envoy/service/auth/v3"
)

// TODO support different repositories
// 1: CSV
// 2: Kubernetes Secret
// 3: HTTP(s) url

var (
	ingressPort   = flag.Int("ingressPort", 50051, "The ingressPort that the server starts listening")
	egressPort    = flag.Int("egressPort", 50052, "The egressPort that the server starts listening")
	pkiAddress    = flag.String("pkiAddress", "", "The address to where the PKI endpoint is available. If omitted, the PKI is searched via Kubernetes Service.")
	caPath        = flag.String("pkiCA", "/ca", "The path of the ca endpoint.")
	csrPath       = flag.String("pkiCSR", "/csr", "The path of the csr endpoint.")
	csvRepository = flag.String("csvRepository", "", "The path to a CSV user repository (with columns 'username', 'password' and 'userId').")
)

func main() {
	flag.Parse()

	pkiConfig := &pki.PKIConfig{
		BaseAddress: *pkiAddress,
		CAPath:      *caPath,
		CSRPath:     *csrPath,
	}

	logrus.SetLevel(logrus.TraceLevel)

	pki.EnsureKeyMaterial(pkiConfig)

	if *csvRepository != "" {
		user_repository.ConfigureCSVRepository(*csvRepository)
	}

	go func() {
		var opts []grpc.ServerOption

		ingressServer := grpc.NewServer(opts...)
		auth.RegisterAuthorizationServer(ingressServer, &translator.IngressServer{})

		listen, err := net.Listen("tcp", fmt.Sprintf(":%v", *ingressPort))
		if err != nil {
			logrus.Fatalf("Failed to listen to ingressPort: %v", *ingressPort)
		}

		logrus.Infof("Starting ingress-server on address :%v", *ingressPort)
		err = ingressServer.Serve(listen)

		if err != nil {
			logrus.Fatalf("Server could not start: %v", err)
			return
		}
	}()

	var opts []grpc.ServerOption

	egressServer := grpc.NewServer(opts...)
	auth.RegisterAuthorizationServer(egressServer, &translator.EgressServer{})

	listen, err := net.Listen("tcp", fmt.Sprintf(":%v", *egressPort))
	if err != nil {
		logrus.Fatalf("Failed to listen to egressPort: %v", *egressPort)
	}

	logrus.Infof("Starting egress-server on address :%v", *egressPort)
	err = egressServer.Serve(listen)

	if err != nil {
		logrus.Fatalf("Server could not start: %v", err)
		return
	}
}
