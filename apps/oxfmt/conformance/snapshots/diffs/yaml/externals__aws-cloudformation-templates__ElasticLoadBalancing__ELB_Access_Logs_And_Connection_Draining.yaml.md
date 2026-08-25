# externals/aws-cloudformation-templates/ElasticLoadBalancing/ELB_Access_Logs_And_Connection_Draining.yaml

> We match Prettier main (prettier/prettier#19764): block scalar trailing whitespace is part of the value; 3.9.6 drops it. See crates/oxc_formatter_yaml/AGENTS.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -258,9 +258,9 @@
                    --region ${AWS::Region}
 
           /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                    --resource WebServerGroup \
-                   --region ${AWS::Region}
+                   --region ${AWS::Region} 
 
   InstanceSecurityGroup:
     Type: AWS::EC2::SecurityGroup
     Properties:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: "AWS CloudFormation Sample Template ELB_Access_Logs_And_Connection_Draining: Creates a load balanced, scalable sample website using Elastic Load Balancer attached to an Auto Scaling group. The ELB has connection draining enabled and also puts access logs into an S3 bucket. ** This template creates one or more Amazon EC2 instances. You will be billed for the AWS resources used if you create a stack from this template."

Metadata:
  License: Apache-2.0

  cfn-lint:
    config:
      regions:
        - us-east-1
        - us-west-2
        - us-west-1
        - eu-west-1
        - eu-central-1
        - ap-southeast-1
        - ap-northeast-1
        - ap-northeast-2
        - ap-southeast-2
        - ap-south-1
        - us-east-2
        - sa-east-1
        - cn-north-1

Parameters:
  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    Default: t3.micro
    ConstraintDescription: must be a valid EC2 instance type.

  KeyName:
    Description: Name of an existing EC2 KeyPair to enable SSH access to the instances
    Type: AWS::EC2::KeyPair::KeyName
    ConstraintDescription: must be the name of an existing EC2 KeyPair.

  SSHLocation:
    Description: The IP address range that can be used to SSH to the EC2 instances
    Type: String
    Default: 0.0.0.0/0
    MinLength: "9"
    MaxLength: "18"
    AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
    ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

  LatestAmiId:
    Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
    Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

Mappings:
  Region2Examples:
    us-east-1:
      Examples: https://s3.amazonaws.com/cloudformation-examples-us-east-1
    us-west-2:
      Examples: https://s3-us-west-2.amazonaws.com/cloudformation-examples-us-west-2
    us-west-1:
      Examples: https://s3-us-west-1.amazonaws.com/cloudformation-examples-us-west-1
    eu-west-1:
      Examples: https://s3-eu-west-1.amazonaws.com/cloudformation-examples-eu-west-1
    eu-central-1:
      Examples: https://s3-eu-central-1.amazonaws.com/cloudformation-examples-eu-central-1
    ap-southeast-1:
      Examples: https://s3-ap-southeast-1.amazonaws.com/cloudformation-examples-ap-southeast-1
    ap-northeast-1:
      Examples: https://s3-ap-northeast-1.amazonaws.com/cloudformation-examples-ap-northeast-1
    ap-northeast-2:
      Examples: https://s3-ap-northeast-2.amazonaws.com/cloudformation-examples-ap-northeast-2
    ap-southeast-2:
      Examples: https://s3-ap-southeast-2.amazonaws.com/cloudformation-examples-ap-southeast-2
    ap-south-1:
      Examples: https://s3-ap-south-1.amazonaws.com/cloudformation-examples-ap-south-1
    us-east-2:
      Examples: https://s3-us-east-2.amazonaws.com/cloudformation-examples-us-east-2
    sa-east-1:
      Examples: https://s3-sa-east-1.amazonaws.com/cloudformation-examples-sa-east-1
    cn-north-1:
      Examples: https://s3.cn-north-1.amazonaws.com.cn/cloudformation-examples-cn-north-1

  Region2ELBAccountId:
    us-east-1:
      AccountId: "127311923021"
    us-west-1:
      AccountId: "027434742980"
    us-west-2:
      AccountId: "797873946194"
    eu-west-1:
      AccountId: "156460612806"
    ap-northeast-1:
      AccountId: "582318560864"
    ap-northeast-2:
      AccountId: "600734575887"
    ap-southeast-1:
      AccountId: "114774131450"
    ap-southeast-2:
      AccountId: "783225319266"
    ap-south-1:
      AccountId: "718504428378"
    us-east-2:
      AccountId: "033677994240"
    sa-east-1:
      AccountId: "507241528517"
    cn-north-1:
      AccountId: "638102146993"
    eu-central-1:
      AccountId: "054676820928"

Resources:
  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
    DependsOn: LogsBucketPolicy
    Properties:
      AvailabilityZones: !GetAZs
      CrossZone: "true"
      Listeners:
        - LoadBalancerPort: "80"
          InstancePort: "80"
          Protocol: HTTP
      HealthCheck:
        Target: HTTP:80/
        HealthyThreshold: "3"
        UnhealthyThreshold: "5"
        Interval: "30"
        Timeout: "5"
      ConnectionDrainingPolicy:
        Enabled: "true"
        Timeout: "300"
      AccessLoggingPolicy:
        S3BucketName: !Ref LogsBucket
        S3BucketPrefix: Logs
        Enabled: "true"
        EmitInterval: "60"
  LogsBucket:
    Type: AWS::S3::Bucket
    DeletionPolicy: Retain
    Properties:
      AccessControl: Private
  LogsBucketPolicy:
    Type: AWS::S3::BucketPolicy
    Properties:
      Bucket: !Ref LogsBucket
      PolicyDocument:
        Version: "2012-10-17"
        Statement:
          - Sid: ELBAccessLogs20130930
            Effect: Allow
            Resource:
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*
            Principal:
              AWS: !FindInMap
                - Region2ELBAccountId
                - !Ref AWS::Region
                - AccountId
            Action:
              - s3:PutObject
          - Action: s3:*
            Condition:
              Bool:
                aws:SecureTransport: false
            Effect: Deny
            Principal:
              AWS: "*"
            Resource:
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*

  WebServerGroup:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    UpdatePolicy:
      AutoScalingRollingUpdate:
        MinInstancesInService: 1
        MaxBatchSize: 1
        PauseTime: PT15M
        WaitOnResourceSignals: true
    Type: AWS::AutoScaling::AutoScalingGroup
    Properties:
      AvailabilityZones: !GetAZs
      LaunchConfigurationName: !Ref LaunchConfig
      MinSize: 2
      MaxSize: 2
      LoadBalancerNames:
        - !Ref ElasticLoadBalancer

  LaunchConfig:
    Type: AWS::AutoScaling::LaunchConfiguration
    Metadata:
      Comment: Install a simple application
      AWS::CloudFormation::Init:
        config:
          packages:
            yum:
              httpd: []
          files:
            /var/www/html/index.html:
              content: !Join
                - ""
                - - <img src="
                  - !FindInMap
                    - Region2Examples
                    - !Ref AWS::Region
                    - Examples
                  - /cloudformation_graphic.png" alt="AWS CloudFormation Logo"/>
                  - <h1>Congratulations, you have successfully launched the AWS CloudFormation sample.</h1>
              mode: "000644"
              owner: root
              group: root
            /etc/cfn/cfn-hup.conf:
              content: !Join
                - ""
                - - "[main] "
                  - stack=
                  - !Ref AWS::StackId
                  - " "
                  - region=
                  - !Ref AWS::Region
                  - " "
              mode: "000400"
              owner: root
              group: root
            /etc/cfn/hooks.d/cfn-auto-reloader.conf:
              content: !Join
                - ""
                - - "[cfn-auto-reloader-hook] "
                  - "triggers=post.update "
                  - "path=Resources.LaunchConfig.Metadata.AWS::CloudFormation::Init "
                  - "action=/opt/aws/bin/cfn-init -v "
                  - "         --stack "
                  - !Ref AWS::StackName
                  - "         --resource LaunchConfig "
                  - "         --region "
                  - !Ref AWS::Region
                  - " "
                  - "runas=root "
          services:
            sysvinit:
              httpd:
                enabled: "true"
                ensureRunning: "true"
              cfn-hup:
                enabled: "true"
                ensureRunning: "true"
                files:
                  - /etc/cfn/cfn-hup.conf
                  - /etc/cfn/hooks.d/cfn-auto-reloader.conf
    Properties:
      KeyName: !Ref KeyName
      ImageId: !Ref LatestAmiId
      SecurityGroups:
        - !Ref InstanceSecurityGroup
      InstanceType: !Ref InstanceType
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource LaunchConfig \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource WebServerGroup \
                   --region ${AWS::Region} 

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the configured port
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: "22"
          ToPort: "22"
          CidrIp: !Ref SSHLocation
        - IpProtocol: tcp
          FromPort: "80"
          ToPort: "80"
          CidrIp: 0.0.0.0/0

Outputs:
  URL:
    Description: URL of the website
    Value: !Join
      - ""
      - - http://
        - !GetAtt ElasticLoadBalancer.DNSName

`````

### Expected (prettier)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: "AWS CloudFormation Sample Template ELB_Access_Logs_And_Connection_Draining: Creates a load balanced, scalable sample website using Elastic Load Balancer attached to an Auto Scaling group. The ELB has connection draining enabled and also puts access logs into an S3 bucket. ** This template creates one or more Amazon EC2 instances. You will be billed for the AWS resources used if you create a stack from this template."

Metadata:
  License: Apache-2.0

  cfn-lint:
    config:
      regions:
        - us-east-1
        - us-west-2
        - us-west-1
        - eu-west-1
        - eu-central-1
        - ap-southeast-1
        - ap-northeast-1
        - ap-northeast-2
        - ap-southeast-2
        - ap-south-1
        - us-east-2
        - sa-east-1
        - cn-north-1

Parameters:
  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    Default: t3.micro
    ConstraintDescription: must be a valid EC2 instance type.

  KeyName:
    Description: Name of an existing EC2 KeyPair to enable SSH access to the instances
    Type: AWS::EC2::KeyPair::KeyName
    ConstraintDescription: must be the name of an existing EC2 KeyPair.

  SSHLocation:
    Description: The IP address range that can be used to SSH to the EC2 instances
    Type: String
    Default: 0.0.0.0/0
    MinLength: "9"
    MaxLength: "18"
    AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
    ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

  LatestAmiId:
    Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
    Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

Mappings:
  Region2Examples:
    us-east-1:
      Examples: https://s3.amazonaws.com/cloudformation-examples-us-east-1
    us-west-2:
      Examples: https://s3-us-west-2.amazonaws.com/cloudformation-examples-us-west-2
    us-west-1:
      Examples: https://s3-us-west-1.amazonaws.com/cloudformation-examples-us-west-1
    eu-west-1:
      Examples: https://s3-eu-west-1.amazonaws.com/cloudformation-examples-eu-west-1
    eu-central-1:
      Examples: https://s3-eu-central-1.amazonaws.com/cloudformation-examples-eu-central-1
    ap-southeast-1:
      Examples: https://s3-ap-southeast-1.amazonaws.com/cloudformation-examples-ap-southeast-1
    ap-northeast-1:
      Examples: https://s3-ap-northeast-1.amazonaws.com/cloudformation-examples-ap-northeast-1
    ap-northeast-2:
      Examples: https://s3-ap-northeast-2.amazonaws.com/cloudformation-examples-ap-northeast-2
    ap-southeast-2:
      Examples: https://s3-ap-southeast-2.amazonaws.com/cloudformation-examples-ap-southeast-2
    ap-south-1:
      Examples: https://s3-ap-south-1.amazonaws.com/cloudformation-examples-ap-south-1
    us-east-2:
      Examples: https://s3-us-east-2.amazonaws.com/cloudformation-examples-us-east-2
    sa-east-1:
      Examples: https://s3-sa-east-1.amazonaws.com/cloudformation-examples-sa-east-1
    cn-north-1:
      Examples: https://s3.cn-north-1.amazonaws.com.cn/cloudformation-examples-cn-north-1

  Region2ELBAccountId:
    us-east-1:
      AccountId: "127311923021"
    us-west-1:
      AccountId: "027434742980"
    us-west-2:
      AccountId: "797873946194"
    eu-west-1:
      AccountId: "156460612806"
    ap-northeast-1:
      AccountId: "582318560864"
    ap-northeast-2:
      AccountId: "600734575887"
    ap-southeast-1:
      AccountId: "114774131450"
    ap-southeast-2:
      AccountId: "783225319266"
    ap-south-1:
      AccountId: "718504428378"
    us-east-2:
      AccountId: "033677994240"
    sa-east-1:
      AccountId: "507241528517"
    cn-north-1:
      AccountId: "638102146993"
    eu-central-1:
      AccountId: "054676820928"

Resources:
  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
    DependsOn: LogsBucketPolicy
    Properties:
      AvailabilityZones: !GetAZs
      CrossZone: "true"
      Listeners:
        - LoadBalancerPort: "80"
          InstancePort: "80"
          Protocol: HTTP
      HealthCheck:
        Target: HTTP:80/
        HealthyThreshold: "3"
        UnhealthyThreshold: "5"
        Interval: "30"
        Timeout: "5"
      ConnectionDrainingPolicy:
        Enabled: "true"
        Timeout: "300"
      AccessLoggingPolicy:
        S3BucketName: !Ref LogsBucket
        S3BucketPrefix: Logs
        Enabled: "true"
        EmitInterval: "60"
  LogsBucket:
    Type: AWS::S3::Bucket
    DeletionPolicy: Retain
    Properties:
      AccessControl: Private
  LogsBucketPolicy:
    Type: AWS::S3::BucketPolicy
    Properties:
      Bucket: !Ref LogsBucket
      PolicyDocument:
        Version: "2012-10-17"
        Statement:
          - Sid: ELBAccessLogs20130930
            Effect: Allow
            Resource:
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*
            Principal:
              AWS: !FindInMap
                - Region2ELBAccountId
                - !Ref AWS::Region
                - AccountId
            Action:
              - s3:PutObject
          - Action: s3:*
            Condition:
              Bool:
                aws:SecureTransport: false
            Effect: Deny
            Principal:
              AWS: "*"
            Resource:
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*

  WebServerGroup:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    UpdatePolicy:
      AutoScalingRollingUpdate:
        MinInstancesInService: 1
        MaxBatchSize: 1
        PauseTime: PT15M
        WaitOnResourceSignals: true
    Type: AWS::AutoScaling::AutoScalingGroup
    Properties:
      AvailabilityZones: !GetAZs
      LaunchConfigurationName: !Ref LaunchConfig
      MinSize: 2
      MaxSize: 2
      LoadBalancerNames:
        - !Ref ElasticLoadBalancer

  LaunchConfig:
    Type: AWS::AutoScaling::LaunchConfiguration
    Metadata:
      Comment: Install a simple application
      AWS::CloudFormation::Init:
        config:
          packages:
            yum:
              httpd: []
          files:
            /var/www/html/index.html:
              content: !Join
                - ""
                - - <img src="
                  - !FindInMap
                    - Region2Examples
                    - !Ref AWS::Region
                    - Examples
                  - /cloudformation_graphic.png" alt="AWS CloudFormation Logo"/>
                  - <h1>Congratulations, you have successfully launched the AWS CloudFormation sample.</h1>
              mode: "000644"
              owner: root
              group: root
            /etc/cfn/cfn-hup.conf:
              content: !Join
                - ""
                - - "[main] "
                  - stack=
                  - !Ref AWS::StackId
                  - " "
                  - region=
                  - !Ref AWS::Region
                  - " "
              mode: "000400"
              owner: root
              group: root
            /etc/cfn/hooks.d/cfn-auto-reloader.conf:
              content: !Join
                - ""
                - - "[cfn-auto-reloader-hook] "
                  - "triggers=post.update "
                  - "path=Resources.LaunchConfig.Metadata.AWS::CloudFormation::Init "
                  - "action=/opt/aws/bin/cfn-init -v "
                  - "         --stack "
                  - !Ref AWS::StackName
                  - "         --resource LaunchConfig "
                  - "         --region "
                  - !Ref AWS::Region
                  - " "
                  - "runas=root "
          services:
            sysvinit:
              httpd:
                enabled: "true"
                ensureRunning: "true"
              cfn-hup:
                enabled: "true"
                ensureRunning: "true"
                files:
                  - /etc/cfn/cfn-hup.conf
                  - /etc/cfn/hooks.d/cfn-auto-reloader.conf
    Properties:
      KeyName: !Ref KeyName
      ImageId: !Ref LatestAmiId
      SecurityGroups:
        - !Ref InstanceSecurityGroup
      InstanceType: !Ref InstanceType
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource LaunchConfig \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource WebServerGroup \
                   --region ${AWS::Region}

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the configured port
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: "22"
          ToPort: "22"
          CidrIp: !Ref SSHLocation
        - IpProtocol: tcp
          FromPort: "80"
          ToPort: "80"
          CidrIp: 0.0.0.0/0

Outputs:
  URL:
    Description: URL of the website
    Value: !Join
      - ""
      - - http://
        - !GetAtt ElasticLoadBalancer.DNSName

`````

## Option 2

`````json
{"printWidth":100,"tabWidth":4,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -264,9 +264,9 @@
                              --region ${AWS::Region}
 
                     /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                              --resource WebServerGroup \
-                             --region ${AWS::Region}
+                             --region ${AWS::Region} 
 
     InstanceSecurityGroup:
         Type: AWS::EC2::SecurityGroup
         Properties:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description:
    "AWS CloudFormation Sample Template ELB_Access_Logs_And_Connection_Draining: Creates a load
    balanced, scalable sample website using Elastic Load Balancer attached to an Auto Scaling group.
    The ELB has connection draining enabled and also puts access logs into an S3 bucket. ** This
    template creates one or more Amazon EC2 instances. You will be billed for the AWS resources used
    if you create a stack from this template."

Metadata:
    License: Apache-2.0

    cfn-lint:
        config:
            regions:
                - us-east-1
                - us-west-2
                - us-west-1
                - eu-west-1
                - eu-central-1
                - ap-southeast-1
                - ap-northeast-1
                - ap-northeast-2
                - ap-southeast-2
                - ap-south-1
                - us-east-2
                - sa-east-1
                - cn-north-1

Parameters:
    InstanceType:
        Description: WebServer EC2 instance type
        Type: String
        Default: t3.micro
        ConstraintDescription: must be a valid EC2 instance type.

    KeyName:
        Description: Name of an existing EC2 KeyPair to enable SSH access to the instances
        Type: AWS::EC2::KeyPair::KeyName
        ConstraintDescription: must be the name of an existing EC2 KeyPair.

    SSHLocation:
        Description: The IP address range that can be used to SSH to the EC2 instances
        Type: String
        Default: 0.0.0.0/0
        MinLength: "9"
        MaxLength: "18"
        AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
        ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

    LatestAmiId:
        Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
        Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

Mappings:
    Region2Examples:
        us-east-1:
            Examples: https://s3.amazonaws.com/cloudformation-examples-us-east-1
        us-west-2:
            Examples: https://s3-us-west-2.amazonaws.com/cloudformation-examples-us-west-2
        us-west-1:
            Examples: https://s3-us-west-1.amazonaws.com/cloudformation-examples-us-west-1
        eu-west-1:
            Examples: https://s3-eu-west-1.amazonaws.com/cloudformation-examples-eu-west-1
        eu-central-1:
            Examples: https://s3-eu-central-1.amazonaws.com/cloudformation-examples-eu-central-1
        ap-southeast-1:
            Examples: https://s3-ap-southeast-1.amazonaws.com/cloudformation-examples-ap-southeast-1
        ap-northeast-1:
            Examples: https://s3-ap-northeast-1.amazonaws.com/cloudformation-examples-ap-northeast-1
        ap-northeast-2:
            Examples: https://s3-ap-northeast-2.amazonaws.com/cloudformation-examples-ap-northeast-2
        ap-southeast-2:
            Examples: https://s3-ap-southeast-2.amazonaws.com/cloudformation-examples-ap-southeast-2
        ap-south-1:
            Examples: https://s3-ap-south-1.amazonaws.com/cloudformation-examples-ap-south-1
        us-east-2:
            Examples: https://s3-us-east-2.amazonaws.com/cloudformation-examples-us-east-2
        sa-east-1:
            Examples: https://s3-sa-east-1.amazonaws.com/cloudformation-examples-sa-east-1
        cn-north-1:
            Examples: https://s3.cn-north-1.amazonaws.com.cn/cloudformation-examples-cn-north-1

    Region2ELBAccountId:
        us-east-1:
            AccountId: "127311923021"
        us-west-1:
            AccountId: "027434742980"
        us-west-2:
            AccountId: "797873946194"
        eu-west-1:
            AccountId: "156460612806"
        ap-northeast-1:
            AccountId: "582318560864"
        ap-northeast-2:
            AccountId: "600734575887"
        ap-southeast-1:
            AccountId: "114774131450"
        ap-southeast-2:
            AccountId: "783225319266"
        ap-south-1:
            AccountId: "718504428378"
        us-east-2:
            AccountId: "033677994240"
        sa-east-1:
            AccountId: "507241528517"
        cn-north-1:
            AccountId: "638102146993"
        eu-central-1:
            AccountId: "054676820928"

Resources:
    ElasticLoadBalancer:
        Type: AWS::ElasticLoadBalancing::LoadBalancer
        DependsOn: LogsBucketPolicy
        Properties:
            AvailabilityZones: !GetAZs
            CrossZone: "true"
            Listeners:
                - LoadBalancerPort: "80"
                  InstancePort: "80"
                  Protocol: HTTP
            HealthCheck:
                Target: HTTP:80/
                HealthyThreshold: "3"
                UnhealthyThreshold: "5"
                Interval: "30"
                Timeout: "5"
            ConnectionDrainingPolicy:
                Enabled: "true"
                Timeout: "300"
            AccessLoggingPolicy:
                S3BucketName: !Ref LogsBucket
                S3BucketPrefix: Logs
                Enabled: "true"
                EmitInterval: "60"
    LogsBucket:
        Type: AWS::S3::Bucket
        DeletionPolicy: Retain
        Properties:
            AccessControl: Private
    LogsBucketPolicy:
        Type: AWS::S3::BucketPolicy
        Properties:
            Bucket: !Ref LogsBucket
            PolicyDocument:
                Version: "2012-10-17"
                Statement:
                    - Sid: ELBAccessLogs20130930
                      Effect: Allow
                      Resource:
                          - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*
                      Principal:
                          AWS: !FindInMap
                              - Region2ELBAccountId
                              - !Ref AWS::Region
                              - AccountId
                      Action:
                          - s3:PutObject
                    - Action: s3:*
                      Condition:
                          Bool:
                              aws:SecureTransport: false
                      Effect: Deny
                      Principal:
                          AWS: "*"
                      Resource:
                          - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}
                          - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*

    WebServerGroup:
        CreationPolicy:
            ResourceSignal:
                Timeout: PT15M
        UpdatePolicy:
            AutoScalingRollingUpdate:
                MinInstancesInService: 1
                MaxBatchSize: 1
                PauseTime: PT15M
                WaitOnResourceSignals: true
        Type: AWS::AutoScaling::AutoScalingGroup
        Properties:
            AvailabilityZones: !GetAZs
            LaunchConfigurationName: !Ref LaunchConfig
            MinSize: 2
            MaxSize: 2
            LoadBalancerNames:
                - !Ref ElasticLoadBalancer

    LaunchConfig:
        Type: AWS::AutoScaling::LaunchConfiguration
        Metadata:
            Comment: Install a simple application
            AWS::CloudFormation::Init:
                config:
                    packages:
                        yum:
                            httpd: []
                    files:
                        /var/www/html/index.html:
                            content: !Join
                                - ""
                                - - <img src="
                                  - !FindInMap
                                    - Region2Examples
                                    - !Ref AWS::Region
                                    - Examples
                                  - /cloudformation_graphic.png" alt="AWS CloudFormation Logo"/>
                                  - <h1>Congratulations, you have successfully launched the AWS
                                    CloudFormation sample.</h1>
                            mode: "000644"
                            owner: root
                            group: root
                        /etc/cfn/cfn-hup.conf:
                            content: !Join
                                - ""
                                - - "[main] "
                                  - stack=
                                  - !Ref AWS::StackId
                                  - " "
                                  - region=
                                  - !Ref AWS::Region
                                  - " "
                            mode: "000400"
                            owner: root
                            group: root
                        /etc/cfn/hooks.d/cfn-auto-reloader.conf:
                            content: !Join
                                - ""
                                - - "[cfn-auto-reloader-hook] "
                                  - "triggers=post.update "
                                  - "path=Resources.LaunchConfig.Metadata.AWS::CloudFormation::Init "
                                  - "action=/opt/aws/bin/cfn-init -v "
                                  - "         --stack "
                                  - !Ref AWS::StackName
                                  - "         --resource LaunchConfig "
                                  - "         --region "
                                  - !Ref AWS::Region
                                  - " "
                                  - "runas=root "
                    services:
                        sysvinit:
                            httpd:
                                enabled: "true"
                                ensureRunning: "true"
                            cfn-hup:
                                enabled: "true"
                                ensureRunning: "true"
                                files:
                                    - /etc/cfn/cfn-hup.conf
                                    - /etc/cfn/hooks.d/cfn-auto-reloader.conf
        Properties:
            KeyName: !Ref KeyName
            ImageId: !Ref LatestAmiId
            SecurityGroups:
                - !Ref InstanceSecurityGroup
            InstanceType: !Ref InstanceType
            UserData: !Base64
                Fn::Sub: |
                    #!/bin/bash -xe          
                    yum update -y aws-cfn-bootstrap 
                    /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                             --resource LaunchConfig \
                             --region ${AWS::Region}

                    /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                             --resource WebServerGroup \
                             --region ${AWS::Region} 

    InstanceSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            GroupDescription: Enable SSH access and HTTP access on the configured port
            SecurityGroupIngress:
                - IpProtocol: tcp
                  FromPort: "22"
                  ToPort: "22"
                  CidrIp: !Ref SSHLocation
                - IpProtocol: tcp
                  FromPort: "80"
                  ToPort: "80"
                  CidrIp: 0.0.0.0/0

Outputs:
    URL:
        Description: URL of the website
        Value: !Join
            - ""
            - - http://
              - !GetAtt ElasticLoadBalancer.DNSName

`````

### Expected (prettier)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description:
    "AWS CloudFormation Sample Template ELB_Access_Logs_And_Connection_Draining: Creates a load
    balanced, scalable sample website using Elastic Load Balancer attached to an Auto Scaling group.
    The ELB has connection draining enabled and also puts access logs into an S3 bucket. ** This
    template creates one or more Amazon EC2 instances. You will be billed for the AWS resources used
    if you create a stack from this template."

Metadata:
    License: Apache-2.0

    cfn-lint:
        config:
            regions:
                - us-east-1
                - us-west-2
                - us-west-1
                - eu-west-1
                - eu-central-1
                - ap-southeast-1
                - ap-northeast-1
                - ap-northeast-2
                - ap-southeast-2
                - ap-south-1
                - us-east-2
                - sa-east-1
                - cn-north-1

Parameters:
    InstanceType:
        Description: WebServer EC2 instance type
        Type: String
        Default: t3.micro
        ConstraintDescription: must be a valid EC2 instance type.

    KeyName:
        Description: Name of an existing EC2 KeyPair to enable SSH access to the instances
        Type: AWS::EC2::KeyPair::KeyName
        ConstraintDescription: must be the name of an existing EC2 KeyPair.

    SSHLocation:
        Description: The IP address range that can be used to SSH to the EC2 instances
        Type: String
        Default: 0.0.0.0/0
        MinLength: "9"
        MaxLength: "18"
        AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
        ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

    LatestAmiId:
        Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
        Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

Mappings:
    Region2Examples:
        us-east-1:
            Examples: https://s3.amazonaws.com/cloudformation-examples-us-east-1
        us-west-2:
            Examples: https://s3-us-west-2.amazonaws.com/cloudformation-examples-us-west-2
        us-west-1:
            Examples: https://s3-us-west-1.amazonaws.com/cloudformation-examples-us-west-1
        eu-west-1:
            Examples: https://s3-eu-west-1.amazonaws.com/cloudformation-examples-eu-west-1
        eu-central-1:
            Examples: https://s3-eu-central-1.amazonaws.com/cloudformation-examples-eu-central-1
        ap-southeast-1:
            Examples: https://s3-ap-southeast-1.amazonaws.com/cloudformation-examples-ap-southeast-1
        ap-northeast-1:
            Examples: https://s3-ap-northeast-1.amazonaws.com/cloudformation-examples-ap-northeast-1
        ap-northeast-2:
            Examples: https://s3-ap-northeast-2.amazonaws.com/cloudformation-examples-ap-northeast-2
        ap-southeast-2:
            Examples: https://s3-ap-southeast-2.amazonaws.com/cloudformation-examples-ap-southeast-2
        ap-south-1:
            Examples: https://s3-ap-south-1.amazonaws.com/cloudformation-examples-ap-south-1
        us-east-2:
            Examples: https://s3-us-east-2.amazonaws.com/cloudformation-examples-us-east-2
        sa-east-1:
            Examples: https://s3-sa-east-1.amazonaws.com/cloudformation-examples-sa-east-1
        cn-north-1:
            Examples: https://s3.cn-north-1.amazonaws.com.cn/cloudformation-examples-cn-north-1

    Region2ELBAccountId:
        us-east-1:
            AccountId: "127311923021"
        us-west-1:
            AccountId: "027434742980"
        us-west-2:
            AccountId: "797873946194"
        eu-west-1:
            AccountId: "156460612806"
        ap-northeast-1:
            AccountId: "582318560864"
        ap-northeast-2:
            AccountId: "600734575887"
        ap-southeast-1:
            AccountId: "114774131450"
        ap-southeast-2:
            AccountId: "783225319266"
        ap-south-1:
            AccountId: "718504428378"
        us-east-2:
            AccountId: "033677994240"
        sa-east-1:
            AccountId: "507241528517"
        cn-north-1:
            AccountId: "638102146993"
        eu-central-1:
            AccountId: "054676820928"

Resources:
    ElasticLoadBalancer:
        Type: AWS::ElasticLoadBalancing::LoadBalancer
        DependsOn: LogsBucketPolicy
        Properties:
            AvailabilityZones: !GetAZs
            CrossZone: "true"
            Listeners:
                - LoadBalancerPort: "80"
                  InstancePort: "80"
                  Protocol: HTTP
            HealthCheck:
                Target: HTTP:80/
                HealthyThreshold: "3"
                UnhealthyThreshold: "5"
                Interval: "30"
                Timeout: "5"
            ConnectionDrainingPolicy:
                Enabled: "true"
                Timeout: "300"
            AccessLoggingPolicy:
                S3BucketName: !Ref LogsBucket
                S3BucketPrefix: Logs
                Enabled: "true"
                EmitInterval: "60"
    LogsBucket:
        Type: AWS::S3::Bucket
        DeletionPolicy: Retain
        Properties:
            AccessControl: Private
    LogsBucketPolicy:
        Type: AWS::S3::BucketPolicy
        Properties:
            Bucket: !Ref LogsBucket
            PolicyDocument:
                Version: "2012-10-17"
                Statement:
                    - Sid: ELBAccessLogs20130930
                      Effect: Allow
                      Resource:
                          - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*
                      Principal:
                          AWS: !FindInMap
                              - Region2ELBAccountId
                              - !Ref AWS::Region
                              - AccountId
                      Action:
                          - s3:PutObject
                    - Action: s3:*
                      Condition:
                          Bool:
                              aws:SecureTransport: false
                      Effect: Deny
                      Principal:
                          AWS: "*"
                      Resource:
                          - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}
                          - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*

    WebServerGroup:
        CreationPolicy:
            ResourceSignal:
                Timeout: PT15M
        UpdatePolicy:
            AutoScalingRollingUpdate:
                MinInstancesInService: 1
                MaxBatchSize: 1
                PauseTime: PT15M
                WaitOnResourceSignals: true
        Type: AWS::AutoScaling::AutoScalingGroup
        Properties:
            AvailabilityZones: !GetAZs
            LaunchConfigurationName: !Ref LaunchConfig
            MinSize: 2
            MaxSize: 2
            LoadBalancerNames:
                - !Ref ElasticLoadBalancer

    LaunchConfig:
        Type: AWS::AutoScaling::LaunchConfiguration
        Metadata:
            Comment: Install a simple application
            AWS::CloudFormation::Init:
                config:
                    packages:
                        yum:
                            httpd: []
                    files:
                        /var/www/html/index.html:
                            content: !Join
                                - ""
                                - - <img src="
                                  - !FindInMap
                                    - Region2Examples
                                    - !Ref AWS::Region
                                    - Examples
                                  - /cloudformation_graphic.png" alt="AWS CloudFormation Logo"/>
                                  - <h1>Congratulations, you have successfully launched the AWS
                                    CloudFormation sample.</h1>
                            mode: "000644"
                            owner: root
                            group: root
                        /etc/cfn/cfn-hup.conf:
                            content: !Join
                                - ""
                                - - "[main] "
                                  - stack=
                                  - !Ref AWS::StackId
                                  - " "
                                  - region=
                                  - !Ref AWS::Region
                                  - " "
                            mode: "000400"
                            owner: root
                            group: root
                        /etc/cfn/hooks.d/cfn-auto-reloader.conf:
                            content: !Join
                                - ""
                                - - "[cfn-auto-reloader-hook] "
                                  - "triggers=post.update "
                                  - "path=Resources.LaunchConfig.Metadata.AWS::CloudFormation::Init "
                                  - "action=/opt/aws/bin/cfn-init -v "
                                  - "         --stack "
                                  - !Ref AWS::StackName
                                  - "         --resource LaunchConfig "
                                  - "         --region "
                                  - !Ref AWS::Region
                                  - " "
                                  - "runas=root "
                    services:
                        sysvinit:
                            httpd:
                                enabled: "true"
                                ensureRunning: "true"
                            cfn-hup:
                                enabled: "true"
                                ensureRunning: "true"
                                files:
                                    - /etc/cfn/cfn-hup.conf
                                    - /etc/cfn/hooks.d/cfn-auto-reloader.conf
        Properties:
            KeyName: !Ref KeyName
            ImageId: !Ref LatestAmiId
            SecurityGroups:
                - !Ref InstanceSecurityGroup
            InstanceType: !Ref InstanceType
            UserData: !Base64
                Fn::Sub: |
                    #!/bin/bash -xe          
                    yum update -y aws-cfn-bootstrap 
                    /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                             --resource LaunchConfig \
                             --region ${AWS::Region}

                    /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                             --resource WebServerGroup \
                             --region ${AWS::Region}

    InstanceSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            GroupDescription: Enable SSH access and HTTP access on the configured port
            SecurityGroupIngress:
                - IpProtocol: tcp
                  FromPort: "22"
                  ToPort: "22"
                  CidrIp: !Ref SSHLocation
                - IpProtocol: tcp
                  FromPort: "80"
                  ToPort: "80"
                  CidrIp: 0.0.0.0/0

Outputs:
    URL:
        Description: URL of the website
        Value: !Join
            - ""
            - - http://
              - !GetAtt ElasticLoadBalancer.DNSName

`````

## Option 3

`````json
{"printWidth":120,"singleQuote":true,"bracketSpacing":false,"trailingComma":"none"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -258,9 +258,9 @@
                    --region ${AWS::Region}
 
           /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                    --resource WebServerGroup \
-                   --region ${AWS::Region}
+                   --region ${AWS::Region} 
 
   InstanceSecurityGroup:
     Type: AWS::EC2::SecurityGroup
     Properties:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: '2010-09-09'

Description: 'AWS CloudFormation Sample Template ELB_Access_Logs_And_Connection_Draining: Creates a load balanced, scalable sample website using Elastic Load Balancer attached to an Auto Scaling group. The ELB has connection draining enabled and also puts access logs into an S3 bucket. ** This template creates one or more Amazon EC2 instances. You will be billed for the AWS resources used if you create a stack from this template.'

Metadata:
  License: Apache-2.0

  cfn-lint:
    config:
      regions:
        - us-east-1
        - us-west-2
        - us-west-1
        - eu-west-1
        - eu-central-1
        - ap-southeast-1
        - ap-northeast-1
        - ap-northeast-2
        - ap-southeast-2
        - ap-south-1
        - us-east-2
        - sa-east-1
        - cn-north-1

Parameters:
  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    Default: t3.micro
    ConstraintDescription: must be a valid EC2 instance type.

  KeyName:
    Description: Name of an existing EC2 KeyPair to enable SSH access to the instances
    Type: AWS::EC2::KeyPair::KeyName
    ConstraintDescription: must be the name of an existing EC2 KeyPair.

  SSHLocation:
    Description: The IP address range that can be used to SSH to the EC2 instances
    Type: String
    Default: 0.0.0.0/0
    MinLength: '9'
    MaxLength: '18'
    AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
    ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

  LatestAmiId:
    Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
    Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

Mappings:
  Region2Examples:
    us-east-1:
      Examples: https://s3.amazonaws.com/cloudformation-examples-us-east-1
    us-west-2:
      Examples: https://s3-us-west-2.amazonaws.com/cloudformation-examples-us-west-2
    us-west-1:
      Examples: https://s3-us-west-1.amazonaws.com/cloudformation-examples-us-west-1
    eu-west-1:
      Examples: https://s3-eu-west-1.amazonaws.com/cloudformation-examples-eu-west-1
    eu-central-1:
      Examples: https://s3-eu-central-1.amazonaws.com/cloudformation-examples-eu-central-1
    ap-southeast-1:
      Examples: https://s3-ap-southeast-1.amazonaws.com/cloudformation-examples-ap-southeast-1
    ap-northeast-1:
      Examples: https://s3-ap-northeast-1.amazonaws.com/cloudformation-examples-ap-northeast-1
    ap-northeast-2:
      Examples: https://s3-ap-northeast-2.amazonaws.com/cloudformation-examples-ap-northeast-2
    ap-southeast-2:
      Examples: https://s3-ap-southeast-2.amazonaws.com/cloudformation-examples-ap-southeast-2
    ap-south-1:
      Examples: https://s3-ap-south-1.amazonaws.com/cloudformation-examples-ap-south-1
    us-east-2:
      Examples: https://s3-us-east-2.amazonaws.com/cloudformation-examples-us-east-2
    sa-east-1:
      Examples: https://s3-sa-east-1.amazonaws.com/cloudformation-examples-sa-east-1
    cn-north-1:
      Examples: https://s3.cn-north-1.amazonaws.com.cn/cloudformation-examples-cn-north-1

  Region2ELBAccountId:
    us-east-1:
      AccountId: '127311923021'
    us-west-1:
      AccountId: '027434742980'
    us-west-2:
      AccountId: '797873946194'
    eu-west-1:
      AccountId: '156460612806'
    ap-northeast-1:
      AccountId: '582318560864'
    ap-northeast-2:
      AccountId: '600734575887'
    ap-southeast-1:
      AccountId: '114774131450'
    ap-southeast-2:
      AccountId: '783225319266'
    ap-south-1:
      AccountId: '718504428378'
    us-east-2:
      AccountId: '033677994240'
    sa-east-1:
      AccountId: '507241528517'
    cn-north-1:
      AccountId: '638102146993'
    eu-central-1:
      AccountId: '054676820928'

Resources:
  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
    DependsOn: LogsBucketPolicy
    Properties:
      AvailabilityZones: !GetAZs
      CrossZone: 'true'
      Listeners:
        - LoadBalancerPort: '80'
          InstancePort: '80'
          Protocol: HTTP
      HealthCheck:
        Target: HTTP:80/
        HealthyThreshold: '3'
        UnhealthyThreshold: '5'
        Interval: '30'
        Timeout: '5'
      ConnectionDrainingPolicy:
        Enabled: 'true'
        Timeout: '300'
      AccessLoggingPolicy:
        S3BucketName: !Ref LogsBucket
        S3BucketPrefix: Logs
        Enabled: 'true'
        EmitInterval: '60'
  LogsBucket:
    Type: AWS::S3::Bucket
    DeletionPolicy: Retain
    Properties:
      AccessControl: Private
  LogsBucketPolicy:
    Type: AWS::S3::BucketPolicy
    Properties:
      Bucket: !Ref LogsBucket
      PolicyDocument:
        Version: '2012-10-17'
        Statement:
          - Sid: ELBAccessLogs20130930
            Effect: Allow
            Resource:
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*
            Principal:
              AWS: !FindInMap
                - Region2ELBAccountId
                - !Ref AWS::Region
                - AccountId
            Action:
              - s3:PutObject
          - Action: s3:*
            Condition:
              Bool:
                aws:SecureTransport: false
            Effect: Deny
            Principal:
              AWS: '*'
            Resource:
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*

  WebServerGroup:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    UpdatePolicy:
      AutoScalingRollingUpdate:
        MinInstancesInService: 1
        MaxBatchSize: 1
        PauseTime: PT15M
        WaitOnResourceSignals: true
    Type: AWS::AutoScaling::AutoScalingGroup
    Properties:
      AvailabilityZones: !GetAZs
      LaunchConfigurationName: !Ref LaunchConfig
      MinSize: 2
      MaxSize: 2
      LoadBalancerNames:
        - !Ref ElasticLoadBalancer

  LaunchConfig:
    Type: AWS::AutoScaling::LaunchConfiguration
    Metadata:
      Comment: Install a simple application
      AWS::CloudFormation::Init:
        config:
          packages:
            yum:
              httpd: []
          files:
            /var/www/html/index.html:
              content: !Join
                - ''
                - - <img src="
                  - !FindInMap
                    - Region2Examples
                    - !Ref AWS::Region
                    - Examples
                  - /cloudformation_graphic.png" alt="AWS CloudFormation Logo"/>
                  - <h1>Congratulations, you have successfully launched the AWS CloudFormation sample.</h1>
              mode: '000644'
              owner: root
              group: root
            /etc/cfn/cfn-hup.conf:
              content: !Join
                - ''
                - - '[main] '
                  - stack=
                  - !Ref AWS::StackId
                  - ' '
                  - region=
                  - !Ref AWS::Region
                  - ' '
              mode: '000400'
              owner: root
              group: root
            /etc/cfn/hooks.d/cfn-auto-reloader.conf:
              content: !Join
                - ''
                - - '[cfn-auto-reloader-hook] '
                  - 'triggers=post.update '
                  - 'path=Resources.LaunchConfig.Metadata.AWS::CloudFormation::Init '
                  - 'action=/opt/aws/bin/cfn-init -v '
                  - '         --stack '
                  - !Ref AWS::StackName
                  - '         --resource LaunchConfig '
                  - '         --region '
                  - !Ref AWS::Region
                  - ' '
                  - 'runas=root '
          services:
            sysvinit:
              httpd:
                enabled: 'true'
                ensureRunning: 'true'
              cfn-hup:
                enabled: 'true'
                ensureRunning: 'true'
                files:
                  - /etc/cfn/cfn-hup.conf
                  - /etc/cfn/hooks.d/cfn-auto-reloader.conf
    Properties:
      KeyName: !Ref KeyName
      ImageId: !Ref LatestAmiId
      SecurityGroups:
        - !Ref InstanceSecurityGroup
      InstanceType: !Ref InstanceType
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource LaunchConfig \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource WebServerGroup \
                   --region ${AWS::Region} 

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the configured port
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: '22'
          ToPort: '22'
          CidrIp: !Ref SSHLocation
        - IpProtocol: tcp
          FromPort: '80'
          ToPort: '80'
          CidrIp: 0.0.0.0/0

Outputs:
  URL:
    Description: URL of the website
    Value: !Join
      - ''
      - - http://
        - !GetAtt ElasticLoadBalancer.DNSName

`````

### Expected (prettier)

`````yaml
AWSTemplateFormatVersion: '2010-09-09'

Description: 'AWS CloudFormation Sample Template ELB_Access_Logs_And_Connection_Draining: Creates a load balanced, scalable sample website using Elastic Load Balancer attached to an Auto Scaling group. The ELB has connection draining enabled and also puts access logs into an S3 bucket. ** This template creates one or more Amazon EC2 instances. You will be billed for the AWS resources used if you create a stack from this template.'

Metadata:
  License: Apache-2.0

  cfn-lint:
    config:
      regions:
        - us-east-1
        - us-west-2
        - us-west-1
        - eu-west-1
        - eu-central-1
        - ap-southeast-1
        - ap-northeast-1
        - ap-northeast-2
        - ap-southeast-2
        - ap-south-1
        - us-east-2
        - sa-east-1
        - cn-north-1

Parameters:
  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    Default: t3.micro
    ConstraintDescription: must be a valid EC2 instance type.

  KeyName:
    Description: Name of an existing EC2 KeyPair to enable SSH access to the instances
    Type: AWS::EC2::KeyPair::KeyName
    ConstraintDescription: must be the name of an existing EC2 KeyPair.

  SSHLocation:
    Description: The IP address range that can be used to SSH to the EC2 instances
    Type: String
    Default: 0.0.0.0/0
    MinLength: '9'
    MaxLength: '18'
    AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
    ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

  LatestAmiId:
    Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
    Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

Mappings:
  Region2Examples:
    us-east-1:
      Examples: https://s3.amazonaws.com/cloudformation-examples-us-east-1
    us-west-2:
      Examples: https://s3-us-west-2.amazonaws.com/cloudformation-examples-us-west-2
    us-west-1:
      Examples: https://s3-us-west-1.amazonaws.com/cloudformation-examples-us-west-1
    eu-west-1:
      Examples: https://s3-eu-west-1.amazonaws.com/cloudformation-examples-eu-west-1
    eu-central-1:
      Examples: https://s3-eu-central-1.amazonaws.com/cloudformation-examples-eu-central-1
    ap-southeast-1:
      Examples: https://s3-ap-southeast-1.amazonaws.com/cloudformation-examples-ap-southeast-1
    ap-northeast-1:
      Examples: https://s3-ap-northeast-1.amazonaws.com/cloudformation-examples-ap-northeast-1
    ap-northeast-2:
      Examples: https://s3-ap-northeast-2.amazonaws.com/cloudformation-examples-ap-northeast-2
    ap-southeast-2:
      Examples: https://s3-ap-southeast-2.amazonaws.com/cloudformation-examples-ap-southeast-2
    ap-south-1:
      Examples: https://s3-ap-south-1.amazonaws.com/cloudformation-examples-ap-south-1
    us-east-2:
      Examples: https://s3-us-east-2.amazonaws.com/cloudformation-examples-us-east-2
    sa-east-1:
      Examples: https://s3-sa-east-1.amazonaws.com/cloudformation-examples-sa-east-1
    cn-north-1:
      Examples: https://s3.cn-north-1.amazonaws.com.cn/cloudformation-examples-cn-north-1

  Region2ELBAccountId:
    us-east-1:
      AccountId: '127311923021'
    us-west-1:
      AccountId: '027434742980'
    us-west-2:
      AccountId: '797873946194'
    eu-west-1:
      AccountId: '156460612806'
    ap-northeast-1:
      AccountId: '582318560864'
    ap-northeast-2:
      AccountId: '600734575887'
    ap-southeast-1:
      AccountId: '114774131450'
    ap-southeast-2:
      AccountId: '783225319266'
    ap-south-1:
      AccountId: '718504428378'
    us-east-2:
      AccountId: '033677994240'
    sa-east-1:
      AccountId: '507241528517'
    cn-north-1:
      AccountId: '638102146993'
    eu-central-1:
      AccountId: '054676820928'

Resources:
  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
    DependsOn: LogsBucketPolicy
    Properties:
      AvailabilityZones: !GetAZs
      CrossZone: 'true'
      Listeners:
        - LoadBalancerPort: '80'
          InstancePort: '80'
          Protocol: HTTP
      HealthCheck:
        Target: HTTP:80/
        HealthyThreshold: '3'
        UnhealthyThreshold: '5'
        Interval: '30'
        Timeout: '5'
      ConnectionDrainingPolicy:
        Enabled: 'true'
        Timeout: '300'
      AccessLoggingPolicy:
        S3BucketName: !Ref LogsBucket
        S3BucketPrefix: Logs
        Enabled: 'true'
        EmitInterval: '60'
  LogsBucket:
    Type: AWS::S3::Bucket
    DeletionPolicy: Retain
    Properties:
      AccessControl: Private
  LogsBucketPolicy:
    Type: AWS::S3::BucketPolicy
    Properties:
      Bucket: !Ref LogsBucket
      PolicyDocument:
        Version: '2012-10-17'
        Statement:
          - Sid: ELBAccessLogs20130930
            Effect: Allow
            Resource:
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*
            Principal:
              AWS: !FindInMap
                - Region2ELBAccountId
                - !Ref AWS::Region
                - AccountId
            Action:
              - s3:PutObject
          - Action: s3:*
            Condition:
              Bool:
                aws:SecureTransport: false
            Effect: Deny
            Principal:
              AWS: '*'
            Resource:
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}
              - !Sub arn:${AWS::Partition}:s3:::${LogsBucket}/Logs/AWSLogs/${AWS::AccountId}/*

  WebServerGroup:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    UpdatePolicy:
      AutoScalingRollingUpdate:
        MinInstancesInService: 1
        MaxBatchSize: 1
        PauseTime: PT15M
        WaitOnResourceSignals: true
    Type: AWS::AutoScaling::AutoScalingGroup
    Properties:
      AvailabilityZones: !GetAZs
      LaunchConfigurationName: !Ref LaunchConfig
      MinSize: 2
      MaxSize: 2
      LoadBalancerNames:
        - !Ref ElasticLoadBalancer

  LaunchConfig:
    Type: AWS::AutoScaling::LaunchConfiguration
    Metadata:
      Comment: Install a simple application
      AWS::CloudFormation::Init:
        config:
          packages:
            yum:
              httpd: []
          files:
            /var/www/html/index.html:
              content: !Join
                - ''
                - - <img src="
                  - !FindInMap
                    - Region2Examples
                    - !Ref AWS::Region
                    - Examples
                  - /cloudformation_graphic.png" alt="AWS CloudFormation Logo"/>
                  - <h1>Congratulations, you have successfully launched the AWS CloudFormation sample.</h1>
              mode: '000644'
              owner: root
              group: root
            /etc/cfn/cfn-hup.conf:
              content: !Join
                - ''
                - - '[main] '
                  - stack=
                  - !Ref AWS::StackId
                  - ' '
                  - region=
                  - !Ref AWS::Region
                  - ' '
              mode: '000400'
              owner: root
              group: root
            /etc/cfn/hooks.d/cfn-auto-reloader.conf:
              content: !Join
                - ''
                - - '[cfn-auto-reloader-hook] '
                  - 'triggers=post.update '
                  - 'path=Resources.LaunchConfig.Metadata.AWS::CloudFormation::Init '
                  - 'action=/opt/aws/bin/cfn-init -v '
                  - '         --stack '
                  - !Ref AWS::StackName
                  - '         --resource LaunchConfig '
                  - '         --region '
                  - !Ref AWS::Region
                  - ' '
                  - 'runas=root '
          services:
            sysvinit:
              httpd:
                enabled: 'true'
                ensureRunning: 'true'
              cfn-hup:
                enabled: 'true'
                ensureRunning: 'true'
                files:
                  - /etc/cfn/cfn-hup.conf
                  - /etc/cfn/hooks.d/cfn-auto-reloader.conf
    Properties:
      KeyName: !Ref KeyName
      ImageId: !Ref LatestAmiId
      SecurityGroups:
        - !Ref InstanceSecurityGroup
      InstanceType: !Ref InstanceType
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource LaunchConfig \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource WebServerGroup \
                   --region ${AWS::Region}

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the configured port
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: '22'
          ToPort: '22'
          CidrIp: !Ref SSHLocation
        - IpProtocol: tcp
          FromPort: '80'
          ToPort: '80'
          CidrIp: 0.0.0.0/0

Outputs:
  URL:
    Description: URL of the website
    Value: !Join
      - ''
      - - http://
        - !GetAtt ElasticLoadBalancer.DNSName

`````
