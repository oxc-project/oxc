# externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBStickinessSample.yaml

> block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -221,9 +221,9 @@
                    --region ${AWS::Region}
 
           /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                    --resource EC2Instance1 \
-                   --region ${AWS::Region}
+                   --region ${AWS::Region} 
 
   EC2Instance2:
     CreationPolicy:
       ResourceSignal:
@@ -245,9 +245,9 @@
                    --region ${AWS::Region}
 
           /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                    --resource EC2Instance2 \
-                   --region ${AWS::Region}
+                   --region ${AWS::Region} 
 
   InstanceSecurityGroup:
     Type: AWS::EC2::SecurityGroup
     Properties:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: "AWS CloudFormation Sample Template ELBStickinessSample: Create a load balanced sample web site with ELB stickiness enabled. The AI is chosen based on the region in which the stack is run. This example creates 2 EC2 instances behind a load balancer with a simple health check. The ec2 instances are untargeted and may be deployed in one or more availaiblity zones. The web site is available on port 80, however, the instances can be configured to listen on any port (8888 by default). **WARNING** This template creates one or more Amazon EC2 instances and an Elastic Load Balancer. You will be billed for the AWS resources used if you create a stack from this template."

Metadata:
  License: Apache-2.0

Parameters:
  LatestAmiId:
    Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
    Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    AllowedValues:
      - t1.micro
      - t2.nano
      - t2.micro
      - t2.small
      - t2.medium
      - t2.large
      - m1.small
      - m1.medium
      - m1.large
      - m1.xlarge
      - m2.xlarge
      - m2.2xlarge
      - m2.4xlarge
      - m3.medium
      - m3.large
      - m3.xlarge
      - m3.2xlarge
      - m4.large
      - m4.xlarge
      - m4.2xlarge
      - m4.4xlarge
      - m4.10xlarge
      - c1.medium
      - c1.xlarge
      - c3.large
      - c3.xlarge
      - c3.2xlarge
      - c3.4xlarge
      - c3.8xlarge
      - c4.large
      - c4.xlarge
      - c4.2xlarge
      - c4.4xlarge
      - c4.8xlarge
      - g2.2xlarge
      - g2.8xlarge
      - r3.large
      - r3.xlarge
      - r3.2xlarge
      - r3.4xlarge
      - r3.8xlarge
      - i2.xlarge
      - i2.2xlarge
      - i2.4xlarge
      - i2.8xlarge
      - d2.xlarge
      - d2.2xlarge
      - d2.4xlarge
      - d2.8xlarge
      - hs1.8xlarge
      - cr1.8xlarge
      - cc2.8xlarge
    Default: t2.small
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

  SubnetId:
    Type: AWS::EC2::Subnet::Id
    Description: The Subnet ID of the subnet in which to place the instance.

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

Resources:
  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
    Properties:
      AvailabilityZones: !GetAZs
      CrossZone: "true"
      Instances:
        - !Ref EC2Instance1
        - !Ref EC2Instance2
      LBCookieStickinessPolicy:
        - PolicyName: myLBPolicy
          CookieExpirationPeriod: "180"
      Listeners:
        - LoadBalancerPort: "80"
          InstancePort: "80"
          Protocol: HTTP
          PolicyNames:
            - myLBPolicy
      HealthCheck:
        Target: HTTP:80/
        HealthyThreshold: "3"
        UnhealthyThreshold: "5"
        Interval: "30"
        Timeout: "5"

  EC2Instance1:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    Type: AWS::EC2::Instance
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
                  - "path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init "
                  - "action=/opt/aws/bin/cfn-init -v "
                  - "         --stack "
                  - !Ref AWS::StackName
                  - "         --resource EC2Instance1 "
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
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      InstanceType: !Ref InstanceType
      ImageId: !Ref LatestAmiId
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region} 

  EC2Instance2:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    Type: AWS::EC2::Instance
    Properties:
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      InstanceType: !Ref InstanceType
      ImageId: !Ref LatestAmiId
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource EC2Instance2 \
                   --region ${AWS::Region} 

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the inbound port
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
    Description: URL of the sample website
    Value: !Join
      - ""
      - - http://
        - !GetAtt ElasticLoadBalancer.DNSName

`````

### Expected (prettier)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: "AWS CloudFormation Sample Template ELBStickinessSample: Create a load balanced sample web site with ELB stickiness enabled. The AI is chosen based on the region in which the stack is run. This example creates 2 EC2 instances behind a load balancer with a simple health check. The ec2 instances are untargeted and may be deployed in one or more availaiblity zones. The web site is available on port 80, however, the instances can be configured to listen on any port (8888 by default). **WARNING** This template creates one or more Amazon EC2 instances and an Elastic Load Balancer. You will be billed for the AWS resources used if you create a stack from this template."

Metadata:
  License: Apache-2.0

Parameters:
  LatestAmiId:
    Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
    Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    AllowedValues:
      - t1.micro
      - t2.nano
      - t2.micro
      - t2.small
      - t2.medium
      - t2.large
      - m1.small
      - m1.medium
      - m1.large
      - m1.xlarge
      - m2.xlarge
      - m2.2xlarge
      - m2.4xlarge
      - m3.medium
      - m3.large
      - m3.xlarge
      - m3.2xlarge
      - m4.large
      - m4.xlarge
      - m4.2xlarge
      - m4.4xlarge
      - m4.10xlarge
      - c1.medium
      - c1.xlarge
      - c3.large
      - c3.xlarge
      - c3.2xlarge
      - c3.4xlarge
      - c3.8xlarge
      - c4.large
      - c4.xlarge
      - c4.2xlarge
      - c4.4xlarge
      - c4.8xlarge
      - g2.2xlarge
      - g2.8xlarge
      - r3.large
      - r3.xlarge
      - r3.2xlarge
      - r3.4xlarge
      - r3.8xlarge
      - i2.xlarge
      - i2.2xlarge
      - i2.4xlarge
      - i2.8xlarge
      - d2.xlarge
      - d2.2xlarge
      - d2.4xlarge
      - d2.8xlarge
      - hs1.8xlarge
      - cr1.8xlarge
      - cc2.8xlarge
    Default: t2.small
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

  SubnetId:
    Type: AWS::EC2::Subnet::Id
    Description: The Subnet ID of the subnet in which to place the instance.

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

Resources:
  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
    Properties:
      AvailabilityZones: !GetAZs
      CrossZone: "true"
      Instances:
        - !Ref EC2Instance1
        - !Ref EC2Instance2
      LBCookieStickinessPolicy:
        - PolicyName: myLBPolicy
          CookieExpirationPeriod: "180"
      Listeners:
        - LoadBalancerPort: "80"
          InstancePort: "80"
          Protocol: HTTP
          PolicyNames:
            - myLBPolicy
      HealthCheck:
        Target: HTTP:80/
        HealthyThreshold: "3"
        UnhealthyThreshold: "5"
        Interval: "30"
        Timeout: "5"

  EC2Instance1:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    Type: AWS::EC2::Instance
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
                  - "path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init "
                  - "action=/opt/aws/bin/cfn-init -v "
                  - "         --stack "
                  - !Ref AWS::StackName
                  - "         --resource EC2Instance1 "
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
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      InstanceType: !Ref InstanceType
      ImageId: !Ref LatestAmiId
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region}

  EC2Instance2:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    Type: AWS::EC2::Instance
    Properties:
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      InstanceType: !Ref InstanceType
      ImageId: !Ref LatestAmiId
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource EC2Instance2 \
                   --region ${AWS::Region}

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the inbound port
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
    Description: URL of the sample website
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
@@ -230,9 +230,9 @@
                              --region ${AWS::Region}
 
                     /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                              --resource EC2Instance1 \
-                             --region ${AWS::Region}
+                             --region ${AWS::Region} 
 
     EC2Instance2:
         CreationPolicy:
             ResourceSignal:
@@ -254,9 +254,9 @@
                              --region ${AWS::Region}
 
                     /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                              --resource EC2Instance2 \
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
    "AWS CloudFormation Sample Template ELBStickinessSample: Create a load balanced sample web site
    with ELB stickiness enabled. The AI is chosen based on the region in which the stack is run.
    This example creates 2 EC2 instances behind a load balancer with a simple health check. The ec2
    instances are untargeted and may be deployed in one or more availaiblity zones. The web site is
    available on port 80, however, the instances can be configured to listen on any port (8888 by
    default). **WARNING** This template creates one or more Amazon EC2 instances and an Elastic Load
    Balancer. You will be billed for the AWS resources used if you create a stack from this
    template."

Metadata:
    License: Apache-2.0

Parameters:
    LatestAmiId:
        Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
        Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

    InstanceType:
        Description: WebServer EC2 instance type
        Type: String
        AllowedValues:
            - t1.micro
            - t2.nano
            - t2.micro
            - t2.small
            - t2.medium
            - t2.large
            - m1.small
            - m1.medium
            - m1.large
            - m1.xlarge
            - m2.xlarge
            - m2.2xlarge
            - m2.4xlarge
            - m3.medium
            - m3.large
            - m3.xlarge
            - m3.2xlarge
            - m4.large
            - m4.xlarge
            - m4.2xlarge
            - m4.4xlarge
            - m4.10xlarge
            - c1.medium
            - c1.xlarge
            - c3.large
            - c3.xlarge
            - c3.2xlarge
            - c3.4xlarge
            - c3.8xlarge
            - c4.large
            - c4.xlarge
            - c4.2xlarge
            - c4.4xlarge
            - c4.8xlarge
            - g2.2xlarge
            - g2.8xlarge
            - r3.large
            - r3.xlarge
            - r3.2xlarge
            - r3.4xlarge
            - r3.8xlarge
            - i2.xlarge
            - i2.2xlarge
            - i2.4xlarge
            - i2.8xlarge
            - d2.xlarge
            - d2.2xlarge
            - d2.4xlarge
            - d2.8xlarge
            - hs1.8xlarge
            - cr1.8xlarge
            - cc2.8xlarge
        Default: t2.small
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

    SubnetId:
        Type: AWS::EC2::Subnet::Id
        Description: The Subnet ID of the subnet in which to place the instance.

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

Resources:
    ElasticLoadBalancer:
        Type: AWS::ElasticLoadBalancing::LoadBalancer
        Properties:
            AvailabilityZones: !GetAZs
            CrossZone: "true"
            Instances:
                - !Ref EC2Instance1
                - !Ref EC2Instance2
            LBCookieStickinessPolicy:
                - PolicyName: myLBPolicy
                  CookieExpirationPeriod: "180"
            Listeners:
                - LoadBalancerPort: "80"
                  InstancePort: "80"
                  Protocol: HTTP
                  PolicyNames:
                      - myLBPolicy
            HealthCheck:
                Target: HTTP:80/
                HealthyThreshold: "3"
                UnhealthyThreshold: "5"
                Interval: "30"
                Timeout: "5"

    EC2Instance1:
        CreationPolicy:
            ResourceSignal:
                Timeout: PT15M
        Type: AWS::EC2::Instance
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
                                  - "path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init "
                                  - "action=/opt/aws/bin/cfn-init -v "
                                  - "         --stack "
                                  - !Ref AWS::StackName
                                  - "         --resource EC2Instance1 "
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
            SubnetId: !Ref SubnetId
            SecurityGroupIds:
                - !GetAtt InstanceSecurityGroup.GroupId
            KeyName: !Ref KeyName
            InstanceType: !Ref InstanceType
            ImageId: !Ref LatestAmiId
            UserData: !Base64
                Fn::Sub: |
                    #!/bin/bash -xe          
                    yum update -y aws-cfn-bootstrap 
                    /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                             --resource EC2Instance1 \
                             --region ${AWS::Region}

                    /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                             --resource EC2Instance1 \
                             --region ${AWS::Region} 

    EC2Instance2:
        CreationPolicy:
            ResourceSignal:
                Timeout: PT15M
        Type: AWS::EC2::Instance
        Properties:
            SubnetId: !Ref SubnetId
            SecurityGroupIds:
                - !GetAtt InstanceSecurityGroup.GroupId
            KeyName: !Ref KeyName
            InstanceType: !Ref InstanceType
            ImageId: !Ref LatestAmiId
            UserData: !Base64
                Fn::Sub: |
                    #!/bin/bash -xe          
                    yum update -y aws-cfn-bootstrap 
                    /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                             --resource EC2Instance1 \
                             --region ${AWS::Region}

                    /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                             --resource EC2Instance2 \
                             --region ${AWS::Region} 

    InstanceSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            GroupDescription: Enable SSH access and HTTP access on the inbound port
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
        Description: URL of the sample website
        Value: !Join
            - ""
            - - http://
              - !GetAtt ElasticLoadBalancer.DNSName

`````

### Expected (prettier)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description:
    "AWS CloudFormation Sample Template ELBStickinessSample: Create a load balanced sample web site
    with ELB stickiness enabled. The AI is chosen based on the region in which the stack is run.
    This example creates 2 EC2 instances behind a load balancer with a simple health check. The ec2
    instances are untargeted and may be deployed in one or more availaiblity zones. The web site is
    available on port 80, however, the instances can be configured to listen on any port (8888 by
    default). **WARNING** This template creates one or more Amazon EC2 instances and an Elastic Load
    Balancer. You will be billed for the AWS resources used if you create a stack from this
    template."

Metadata:
    License: Apache-2.0

Parameters:
    LatestAmiId:
        Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
        Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

    InstanceType:
        Description: WebServer EC2 instance type
        Type: String
        AllowedValues:
            - t1.micro
            - t2.nano
            - t2.micro
            - t2.small
            - t2.medium
            - t2.large
            - m1.small
            - m1.medium
            - m1.large
            - m1.xlarge
            - m2.xlarge
            - m2.2xlarge
            - m2.4xlarge
            - m3.medium
            - m3.large
            - m3.xlarge
            - m3.2xlarge
            - m4.large
            - m4.xlarge
            - m4.2xlarge
            - m4.4xlarge
            - m4.10xlarge
            - c1.medium
            - c1.xlarge
            - c3.large
            - c3.xlarge
            - c3.2xlarge
            - c3.4xlarge
            - c3.8xlarge
            - c4.large
            - c4.xlarge
            - c4.2xlarge
            - c4.4xlarge
            - c4.8xlarge
            - g2.2xlarge
            - g2.8xlarge
            - r3.large
            - r3.xlarge
            - r3.2xlarge
            - r3.4xlarge
            - r3.8xlarge
            - i2.xlarge
            - i2.2xlarge
            - i2.4xlarge
            - i2.8xlarge
            - d2.xlarge
            - d2.2xlarge
            - d2.4xlarge
            - d2.8xlarge
            - hs1.8xlarge
            - cr1.8xlarge
            - cc2.8xlarge
        Default: t2.small
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

    SubnetId:
        Type: AWS::EC2::Subnet::Id
        Description: The Subnet ID of the subnet in which to place the instance.

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

Resources:
    ElasticLoadBalancer:
        Type: AWS::ElasticLoadBalancing::LoadBalancer
        Properties:
            AvailabilityZones: !GetAZs
            CrossZone: "true"
            Instances:
                - !Ref EC2Instance1
                - !Ref EC2Instance2
            LBCookieStickinessPolicy:
                - PolicyName: myLBPolicy
                  CookieExpirationPeriod: "180"
            Listeners:
                - LoadBalancerPort: "80"
                  InstancePort: "80"
                  Protocol: HTTP
                  PolicyNames:
                      - myLBPolicy
            HealthCheck:
                Target: HTTP:80/
                HealthyThreshold: "3"
                UnhealthyThreshold: "5"
                Interval: "30"
                Timeout: "5"

    EC2Instance1:
        CreationPolicy:
            ResourceSignal:
                Timeout: PT15M
        Type: AWS::EC2::Instance
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
                                  - "path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init "
                                  - "action=/opt/aws/bin/cfn-init -v "
                                  - "         --stack "
                                  - !Ref AWS::StackName
                                  - "         --resource EC2Instance1 "
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
            SubnetId: !Ref SubnetId
            SecurityGroupIds:
                - !GetAtt InstanceSecurityGroup.GroupId
            KeyName: !Ref KeyName
            InstanceType: !Ref InstanceType
            ImageId: !Ref LatestAmiId
            UserData: !Base64
                Fn::Sub: |
                    #!/bin/bash -xe          
                    yum update -y aws-cfn-bootstrap 
                    /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                             --resource EC2Instance1 \
                             --region ${AWS::Region}

                    /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                             --resource EC2Instance1 \
                             --region ${AWS::Region}

    EC2Instance2:
        CreationPolicy:
            ResourceSignal:
                Timeout: PT15M
        Type: AWS::EC2::Instance
        Properties:
            SubnetId: !Ref SubnetId
            SecurityGroupIds:
                - !GetAtt InstanceSecurityGroup.GroupId
            KeyName: !Ref KeyName
            InstanceType: !Ref InstanceType
            ImageId: !Ref LatestAmiId
            UserData: !Base64
                Fn::Sub: |
                    #!/bin/bash -xe          
                    yum update -y aws-cfn-bootstrap 
                    /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                             --resource EC2Instance1 \
                             --region ${AWS::Region}

                    /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                             --resource EC2Instance2 \
                             --region ${AWS::Region}

    InstanceSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            GroupDescription: Enable SSH access and HTTP access on the inbound port
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
        Description: URL of the sample website
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
@@ -221,9 +221,9 @@
                    --region ${AWS::Region}
 
           /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                    --resource EC2Instance1 \
-                   --region ${AWS::Region}
+                   --region ${AWS::Region} 
 
   EC2Instance2:
     CreationPolicy:
       ResourceSignal:
@@ -245,9 +245,9 @@
                    --region ${AWS::Region}
 
           /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                    --resource EC2Instance2 \
-                   --region ${AWS::Region}
+                   --region ${AWS::Region} 
 
   InstanceSecurityGroup:
     Type: AWS::EC2::SecurityGroup
     Properties:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: '2010-09-09'

Description: 'AWS CloudFormation Sample Template ELBStickinessSample: Create a load balanced sample web site with ELB stickiness enabled. The AI is chosen based on the region in which the stack is run. This example creates 2 EC2 instances behind a load balancer with a simple health check. The ec2 instances are untargeted and may be deployed in one or more availaiblity zones. The web site is available on port 80, however, the instances can be configured to listen on any port (8888 by default). **WARNING** This template creates one or more Amazon EC2 instances and an Elastic Load Balancer. You will be billed for the AWS resources used if you create a stack from this template.'

Metadata:
  License: Apache-2.0

Parameters:
  LatestAmiId:
    Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
    Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    AllowedValues:
      - t1.micro
      - t2.nano
      - t2.micro
      - t2.small
      - t2.medium
      - t2.large
      - m1.small
      - m1.medium
      - m1.large
      - m1.xlarge
      - m2.xlarge
      - m2.2xlarge
      - m2.4xlarge
      - m3.medium
      - m3.large
      - m3.xlarge
      - m3.2xlarge
      - m4.large
      - m4.xlarge
      - m4.2xlarge
      - m4.4xlarge
      - m4.10xlarge
      - c1.medium
      - c1.xlarge
      - c3.large
      - c3.xlarge
      - c3.2xlarge
      - c3.4xlarge
      - c3.8xlarge
      - c4.large
      - c4.xlarge
      - c4.2xlarge
      - c4.4xlarge
      - c4.8xlarge
      - g2.2xlarge
      - g2.8xlarge
      - r3.large
      - r3.xlarge
      - r3.2xlarge
      - r3.4xlarge
      - r3.8xlarge
      - i2.xlarge
      - i2.2xlarge
      - i2.4xlarge
      - i2.8xlarge
      - d2.xlarge
      - d2.2xlarge
      - d2.4xlarge
      - d2.8xlarge
      - hs1.8xlarge
      - cr1.8xlarge
      - cc2.8xlarge
    Default: t2.small
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

  SubnetId:
    Type: AWS::EC2::Subnet::Id
    Description: The Subnet ID of the subnet in which to place the instance.

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

Resources:
  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
    Properties:
      AvailabilityZones: !GetAZs
      CrossZone: 'true'
      Instances:
        - !Ref EC2Instance1
        - !Ref EC2Instance2
      LBCookieStickinessPolicy:
        - PolicyName: myLBPolicy
          CookieExpirationPeriod: '180'
      Listeners:
        - LoadBalancerPort: '80'
          InstancePort: '80'
          Protocol: HTTP
          PolicyNames:
            - myLBPolicy
      HealthCheck:
        Target: HTTP:80/
        HealthyThreshold: '3'
        UnhealthyThreshold: '5'
        Interval: '30'
        Timeout: '5'

  EC2Instance1:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    Type: AWS::EC2::Instance
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
                  - 'path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init '
                  - 'action=/opt/aws/bin/cfn-init -v '
                  - '         --stack '
                  - !Ref AWS::StackName
                  - '         --resource EC2Instance1 '
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
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      InstanceType: !Ref InstanceType
      ImageId: !Ref LatestAmiId
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region} 

  EC2Instance2:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    Type: AWS::EC2::Instance
    Properties:
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      InstanceType: !Ref InstanceType
      ImageId: !Ref LatestAmiId
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource EC2Instance2 \
                   --region ${AWS::Region} 

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the inbound port
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
    Description: URL of the sample website
    Value: !Join
      - ''
      - - http://
        - !GetAtt ElasticLoadBalancer.DNSName

`````

### Expected (prettier)

`````yaml
AWSTemplateFormatVersion: '2010-09-09'

Description: 'AWS CloudFormation Sample Template ELBStickinessSample: Create a load balanced sample web site with ELB stickiness enabled. The AI is chosen based on the region in which the stack is run. This example creates 2 EC2 instances behind a load balancer with a simple health check. The ec2 instances are untargeted and may be deployed in one or more availaiblity zones. The web site is available on port 80, however, the instances can be configured to listen on any port (8888 by default). **WARNING** This template creates one or more Amazon EC2 instances and an Elastic Load Balancer. You will be billed for the AWS resources used if you create a stack from this template.'

Metadata:
  License: Apache-2.0

Parameters:
  LatestAmiId:
    Type: AWS::SSM::Parameter::Value<AWS::EC2::Image::Id>
    Default: /aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2

  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    AllowedValues:
      - t1.micro
      - t2.nano
      - t2.micro
      - t2.small
      - t2.medium
      - t2.large
      - m1.small
      - m1.medium
      - m1.large
      - m1.xlarge
      - m2.xlarge
      - m2.2xlarge
      - m2.4xlarge
      - m3.medium
      - m3.large
      - m3.xlarge
      - m3.2xlarge
      - m4.large
      - m4.xlarge
      - m4.2xlarge
      - m4.4xlarge
      - m4.10xlarge
      - c1.medium
      - c1.xlarge
      - c3.large
      - c3.xlarge
      - c3.2xlarge
      - c3.4xlarge
      - c3.8xlarge
      - c4.large
      - c4.xlarge
      - c4.2xlarge
      - c4.4xlarge
      - c4.8xlarge
      - g2.2xlarge
      - g2.8xlarge
      - r3.large
      - r3.xlarge
      - r3.2xlarge
      - r3.4xlarge
      - r3.8xlarge
      - i2.xlarge
      - i2.2xlarge
      - i2.4xlarge
      - i2.8xlarge
      - d2.xlarge
      - d2.2xlarge
      - d2.4xlarge
      - d2.8xlarge
      - hs1.8xlarge
      - cr1.8xlarge
      - cc2.8xlarge
    Default: t2.small
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

  SubnetId:
    Type: AWS::EC2::Subnet::Id
    Description: The Subnet ID of the subnet in which to place the instance.

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

Resources:
  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
    Properties:
      AvailabilityZones: !GetAZs
      CrossZone: 'true'
      Instances:
        - !Ref EC2Instance1
        - !Ref EC2Instance2
      LBCookieStickinessPolicy:
        - PolicyName: myLBPolicy
          CookieExpirationPeriod: '180'
      Listeners:
        - LoadBalancerPort: '80'
          InstancePort: '80'
          Protocol: HTTP
          PolicyNames:
            - myLBPolicy
      HealthCheck:
        Target: HTTP:80/
        HealthyThreshold: '3'
        UnhealthyThreshold: '5'
        Interval: '30'
        Timeout: '5'

  EC2Instance1:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    Type: AWS::EC2::Instance
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
                  - 'path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init '
                  - 'action=/opt/aws/bin/cfn-init -v '
                  - '         --stack '
                  - !Ref AWS::StackName
                  - '         --resource EC2Instance1 '
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
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      InstanceType: !Ref InstanceType
      ImageId: !Ref LatestAmiId
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region}

  EC2Instance2:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
    Type: AWS::EC2::Instance
    Properties:
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      InstanceType: !Ref InstanceType
      ImageId: !Ref LatestAmiId
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe          
          yum update -y aws-cfn-bootstrap 
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackName} \
                   --resource EC2Instance1 \
                   --region ${AWS::Region}

          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                   --resource EC2Instance2 \
                   --region ${AWS::Region}

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the inbound port
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
    Description: URL of the sample website
    Value: !Join
      - ''
      - - http://
        - !GetAtt ElasticLoadBalancer.DNSName

`````
