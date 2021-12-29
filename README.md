# WirePact K8s Basic Auth Translator

This is a "translator" for the WirePact distributed authentication mesh system.
It converts HTTP Basic Auth credentials ([RFC7617](https://tools.ietf.org/html/rfc7617))
to the WirePact common language format (signed JWT) and back.

The list of valid users must be in a CSV file with three columns. The first column
must contain the user id, the second the username and the last column must contain the
password for the user. With this CSV "repository", the translator is able
to convert an outgoing communication (egress) to a signed JWT and the incoming communication
(ingress) back to username/password combination.

Another valid repository is a Kubernetes secret. The data in the secret must be in the form of:
`userid` as key, and the encoded basic value (`username:password`) as value.
An example secret could look like:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: api-credentials
type: Opaque
data:
  123456789: ZFhObGNqcHdZWE56
```

The configuration is done via environmental variables:

- `CSV_PATH`: The path to the csv file
- `KUBERNETES_SECRET`: The name of the Kubernetes secret to use for user repository
- `PKI_ADDRESS`: The address of the available WirePact PKI
- `COMMON_NAME`: The common name for the translator in the signed JWT and certificates
- `INGRESS_PORT`: Ingress communication port (default: 50051)
- `EGRESS_PORT`: Egress communication port (default: 50052)
