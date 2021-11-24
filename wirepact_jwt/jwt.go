package wirepact_jwt

import (
	"time"

	"github.com/golang-jwt/jwt"
	"github.com/sirupsen/logrus"
	"wirepact.ch/k8s-basic-auth-translator/pki"
)

// https://datatracker.ietf.org/doc/html/rfc7515#section-4.1.6

func CreateSignedJWTForUser(userID string) string {
	claims := &jwt.StandardClaims{
		Issuer:    "k8s-basic-auth-translator",
		Audience:  "WirePact",
		IssuedAt:  time.Now().UTC().Unix(),
		ExpiresAt: time.Now().UTC().Add(60 * time.Second).Unix(),
		NotBefore: 0,
		Subject:   userID,
	}

	token := jwt.NewWithClaims(jwt.SigningMethodRS256, claims)
	x5c, x5t := pki.GetJWTCertificateHeaders()
	token.Header["x5c"] = x5c
	token.Header["x5t"] = x5t

	signedToken, err := token.SignedString(pki.GetPrivateKey())
	if err != nil {
		logrus.WithError(err).Fatalf("Could not sign token for user '%v'.", userID)
	}

	return signedToken
}
