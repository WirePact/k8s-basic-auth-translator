package pki

import (
	"bytes"
	"crypto/rand"
	"crypto/rsa"
	"crypto/sha256"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/base64"
	"encoding/pem"
	"net/http"
	"os"

	"github.com/sirupsen/logrus"
)

const (
	caFilename   = "ca.crt"
	certFilename = "cert.crt"
	keyFilename  = "cert.key"
)

var ca *x509.Certificate
var certificate *x509.Certificate
var privateKey *rsa.PrivateKey

func EnsureKeyMaterial(config *PKIConfig) {
	if config.BaseAddress == "" {
		logrus.Debugln("Fetch key material from Kubernetes.")
	} else {
		logrus.Debugf("Fetch key material from %v.", config.BaseAddress)
	}

	loadCA(config)
	loadLocalCert(config)
}

func GetPrivateKey() *rsa.PrivateKey {
	return privateKey
}

func GetJWTCertificateHeaders() ([]string, string) {
	signature := sha256.Sum256(certificate.Raw)
	return []string{
			base64.StdEncoding.EncodeToString(certificate.Raw),
			base64.StdEncoding.EncodeToString(ca.Raw),
		},
		base64.StdEncoding.EncodeToString(signature[:])
}

func GetPublicKeyBase64() string {

	publicKeyDer, _ := x509.MarshalPKIXPublicKey(certificate.PublicKey)
	//_ := pem.Block{
	//	Type:  "CERTIFICATE",
	//	Bytes: publicKeyDer,
	//}

	return base64.StdEncoding.EncodeToString(publicKeyDer)
}

func GetCABase64() string {
	publicKeyDer, _ := x509.MarshalPKIXPublicKey(ca.PublicKey)
	publicKeyBlock := pem.Block{
		Type:  "PUBLIC KEY",
		Bytes: publicKeyDer,
	}

	return base64.StdEncoding.EncodeToString(pem.EncodeToMemory(&publicKeyBlock))
}

func loadCA(config *PKIConfig) {
	if _, err := os.Stat(caFilename); err != nil {
		logrus.Debugln("CA certificate not found. Downloading it from PKI.")

		response, err := http.Get(config.caAddress())
		if err != nil {
			logrus.WithError(err).Fatalln("Could not download CA file from PKI.")
		}

		caFile, err := os.Create(caFilename)
		if err != nil {
			logrus.WithError(err).Fatalln("Could not create ca.crt file.")
		}

		_, _ = caFile.ReadFrom(response.Body)
		_ = caFile.Close()
		_ = response.Body.Close()
	}

	logrus.Debugln("Load ca.crt into memory.")
	certPEMBlock, err := os.ReadFile(caFilename)
	if err != nil {
		logrus.WithError(err).Fatalln("Could not load ca.crt file.")
	}

	certBlock, _ := pem.Decode(certPEMBlock)
	logrus.Debugf("Loaded pem block with type %v and length %v bytes.", certBlock.Type, len(certBlock.Bytes))

	ca, err = x509.ParseCertificate(certBlock.Bytes)
	if err != nil {
		logrus.WithError(err).Fatalln("Could not parse ca certificate.")
	}

	logrus.Infof("Successfully loaded CA certificate with subject %v.", ca.Subject.String())
}

func loadLocalCert(config *PKIConfig) {
	if _, err := os.Stat(keyFilename); err != nil {
		logrus.Debugln("Key does not exist, create one.")
		privateKey, _ = rsa.GenerateKey(rand.Reader, 2048)
		keyOut := pem.EncodeToMemory(&pem.Block{Type: "RSA PRIVATE KEY", Bytes: x509.MarshalPKCS1PrivateKey(privateKey)})
		logrus.Traceln("Wrote Private Key bytes.")

		keyFile, err := os.Create(keyFilename)
		if err != nil {
			logrus.WithError(err).Fatalf("Could not create key file '%v'.", keyFilename)
		}
		_, _ = keyFile.Write(keyOut)
		_ = keyFile.Close()
	} else {
		keyPEMBlock, err := os.ReadFile(keyFilename)
		if err != nil {
			logrus.WithError(err).Fatalf("Could not load %v file.", keyFilename)
		}

		keyBlock, _ := pem.Decode(keyPEMBlock)
		logrus.Debugf("Loaded pem block with type %v and length %v bytes.", keyBlock.Type, len(keyBlock.Bytes))

		privateKey, err = x509.ParsePKCS1PrivateKey(keyBlock.Bytes)
		if err != nil {
			logrus.WithError(err).Fatalf("Could not load %v file.", keyFilename)
		}
	}

	if _, err := os.Stat(certFilename); err != nil {
		logrus.Debugln("Cert does not exist, create CSR and fetch cert from PKI.")

		csr := x509.CertificateRequest{
			Subject: pkix.Name{
				Organization: []string{"WirePact PKI", "K8s Basic Auth Translator"},
				CommonName:   "k8s basic auth translator", // TODO add pod name or something.
			},
			SignatureAlgorithm: x509.SHA256WithRSA,
		}

		csrBytes, err := x509.CreateCertificateRequest(rand.Reader, &csr, privateKey)
		if err != nil {
			logrus.WithError(err).Fatalln("Could not create CSR.")
		}

		csrBuffer := &bytes.Buffer{}
		_ = pem.Encode(csrBuffer, &pem.Block{Type: "CERTIFICATE REQUEST", Bytes: csrBytes})

		response, err := http.Post(config.csrAddress(), "application/pkcs10", csrBuffer)
		if err != nil {
			logrus.WithError(err).Fatalln("Could not send CSR to PKI.")
		}

		certFile, err := os.Create(certFilename)
		if err != nil {
			logrus.WithError(err).Fatalln("Could not create ca.crt file.")
		}

		_, _ = certFile.ReadFrom(response.Body)
		_ = certFile.Close()
		_ = response.Body.Close()
	}

	certPEMBlock, err := os.ReadFile(certFilename)
	if err != nil {
		logrus.WithError(err).Fatalf("Could not load %v file.", certFilename)
	}

	certBlock, _ := pem.Decode(certPEMBlock)
	logrus.Debugf("Loaded pem block with type %v and length %v bytes.", certBlock.Type, len(certBlock.Bytes))

	certificate, err = x509.ParseCertificate(certBlock.Bytes)
	if err != nil {
		logrus.WithError(err).Fatalf("Could not load %v file.", certFilename)
	}

	logrus.Infof("Successfully loaded client certificate from issuer %v to subject %v with serialnumber %v.",
		certificate.Issuer.String(),
		certificate.Subject.String(),
		certificate.SerialNumber)
}
