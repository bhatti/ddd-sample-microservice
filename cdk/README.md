## Setting up CDK

### Install CDK
```bash
npm install -g typescript
npm install aws-cdk-lib
npm install -g aws-cdk
```

### Building CDK App
```bash
npm run build
```

### Configure AWS Account
e.g.  ~/.aws/credentials
```
[default]
aws_access_key_id = AAA
aws_session_token = BBB
aws_secret_access_key = CCC
```

### Listing stacks
```bash
cdk ls
```

### Synthesize an AWS CloudFormation template
```bash
cdk synth
```

### Deploying the stack
```bash
cdk deploy
```

### Destroying the app's resources
```bash
cdk destroy
```
