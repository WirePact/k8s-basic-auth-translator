package certificates

import (
	"fmt"

	"wirepact.ch/k8s-basic-auth-translator/client"
)

func EnsureSigningCertificate() {
	// TODO check where the cert is.

	lol := client.GetKubernetesClient()

}
