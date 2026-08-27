# externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBWithLockedDownAutoScaledInstances.yaml

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
@@ -207,9 +207,9 @@
                    --region ${AWS::Region}
 
           /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                    --resource WebServerGroup \
-                   --region ${AWS::Region}
+                   --region ${AWS::Region} 
 
   ElasticLoadBalancer:
     Type: AWS::ElasticLoadBalancing::LoadBalancer
     Properties:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: "AWS CloudFormation Sample Template ELBWithLockedDownAutoScaledInstances: Create a load balanced, Auto Scaled sample website where the instances are locked down to only accept traffic from the load balancer. This example creates an Auto Scaling group behind a load balancer with a simple health check. The web site is available on port 80, however, the instances can be configured to listen on any port (8888 by default). **WARNING** This template creates one or more Amazon EC2 instances and an Elastic Load Balancer. You will be billed for the AWS resources used if you create a stack from this template."

Metadata:
  License: Apache-2.0

Parameters:
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

Resources:
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
      MinSize: "2"
      MaxSize: "2"
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
      ImageId: !Ref LatestAmiId
      SecurityGroups:
        - !Ref InstanceSecurityGroup
      InstanceType: !Ref InstanceType
      KeyName: !Ref KeyName
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

  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
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

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the inbound port
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: "80"
          ToPort: "80"
          SourceSecurityGroupOwnerId: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.OwnerAlias
          SourceSecurityGroupName: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.GroupName
        - IpProtocol: tcp
          FromPort: "22"
          ToPort: "22"
          CidrIp: !Ref SSHLocation

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

Description: "AWS CloudFormation Sample Template ELBWithLockedDownAutoScaledInstances: Create a load balanced, Auto Scaled sample website where the instances are locked down to only accept traffic from the load balancer. This example creates an Auto Scaling group behind a load balancer with a simple health check. The web site is available on port 80, however, the instances can be configured to listen on any port (8888 by default). **WARNING** This template creates one or more Amazon EC2 instances and an Elastic Load Balancer. You will be billed for the AWS resources used if you create a stack from this template."

Metadata:
  License: Apache-2.0

Parameters:
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

Resources:
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
      MinSize: "2"
      MaxSize: "2"
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
      ImageId: !Ref LatestAmiId
      SecurityGroups:
        - !Ref InstanceSecurityGroup
      InstanceType: !Ref InstanceType
      KeyName: !Ref KeyName
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

  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
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

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the inbound port
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: "80"
          ToPort: "80"
          SourceSecurityGroupOwnerId: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.OwnerAlias
          SourceSecurityGroupName: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.GroupName
        - IpProtocol: tcp
          FromPort: "22"
          ToPort: "22"
          CidrIp: !Ref SSHLocation

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
@@ -215,9 +215,9 @@
                              --region ${AWS::Region}
 
                     /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                              --resource WebServerGroup \
-                             --region ${AWS::Region}
+                             --region ${AWS::Region} 
 
     ElasticLoadBalancer:
         Type: AWS::ElasticLoadBalancing::LoadBalancer
         Properties:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description:
    "AWS CloudFormation Sample Template ELBWithLockedDownAutoScaledInstances: Create a load
    balanced, Auto Scaled sample website where the instances are locked down to only accept traffic
    from the load balancer. This example creates an Auto Scaling group behind a load balancer with a
    simple health check. The web site is available on port 80, however, the instances can be
    configured to listen on any port (8888 by default). **WARNING** This template creates one or
    more Amazon EC2 instances and an Elastic Load Balancer. You will be billed for the AWS resources
    used if you create a stack from this template."

Metadata:
    License: Apache-2.0

Parameters:
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

Resources:
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
            MinSize: "2"
            MaxSize: "2"
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
            ImageId: !Ref LatestAmiId
            SecurityGroups:
                - !Ref InstanceSecurityGroup
            InstanceType: !Ref InstanceType
            KeyName: !Ref KeyName
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

    ElasticLoadBalancer:
        Type: AWS::ElasticLoadBalancing::LoadBalancer
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

    InstanceSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            GroupDescription: Enable SSH access and HTTP access on the inbound port
            SecurityGroupIngress:
                - IpProtocol: tcp
                  FromPort: "80"
                  ToPort: "80"
                  SourceSecurityGroupOwnerId: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.OwnerAlias
                  SourceSecurityGroupName: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.GroupName
                - IpProtocol: tcp
                  FromPort: "22"
                  ToPort: "22"
                  CidrIp: !Ref SSHLocation

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
    "AWS CloudFormation Sample Template ELBWithLockedDownAutoScaledInstances: Create a load
    balanced, Auto Scaled sample website where the instances are locked down to only accept traffic
    from the load balancer. This example creates an Auto Scaling group behind a load balancer with a
    simple health check. The web site is available on port 80, however, the instances can be
    configured to listen on any port (8888 by default). **WARNING** This template creates one or
    more Amazon EC2 instances and an Elastic Load Balancer. You will be billed for the AWS resources
    used if you create a stack from this template."

Metadata:
    License: Apache-2.0

Parameters:
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

Resources:
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
            MinSize: "2"
            MaxSize: "2"
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
            ImageId: !Ref LatestAmiId
            SecurityGroups:
                - !Ref InstanceSecurityGroup
            InstanceType: !Ref InstanceType
            KeyName: !Ref KeyName
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

    ElasticLoadBalancer:
        Type: AWS::ElasticLoadBalancing::LoadBalancer
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

    InstanceSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            GroupDescription: Enable SSH access and HTTP access on the inbound port
            SecurityGroupIngress:
                - IpProtocol: tcp
                  FromPort: "80"
                  ToPort: "80"
                  SourceSecurityGroupOwnerId: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.OwnerAlias
                  SourceSecurityGroupName: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.GroupName
                - IpProtocol: tcp
                  FromPort: "22"
                  ToPort: "22"
                  CidrIp: !Ref SSHLocation

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
@@ -207,9 +207,9 @@
                    --region ${AWS::Region}
 
           /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackName} \
                    --resource WebServerGroup \
-                   --region ${AWS::Region}
+                   --region ${AWS::Region} 
 
   ElasticLoadBalancer:
     Type: AWS::ElasticLoadBalancing::LoadBalancer
     Properties:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: '2010-09-09'

Description: 'AWS CloudFormation Sample Template ELBWithLockedDownAutoScaledInstances: Create a load balanced, Auto Scaled sample website where the instances are locked down to only accept traffic from the load balancer. This example creates an Auto Scaling group behind a load balancer with a simple health check. The web site is available on port 80, however, the instances can be configured to listen on any port (8888 by default). **WARNING** This template creates one or more Amazon EC2 instances and an Elastic Load Balancer. You will be billed for the AWS resources used if you create a stack from this template.'

Metadata:
  License: Apache-2.0

Parameters:
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

Resources:
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
      MinSize: '2'
      MaxSize: '2'
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
      ImageId: !Ref LatestAmiId
      SecurityGroups:
        - !Ref InstanceSecurityGroup
      InstanceType: !Ref InstanceType
      KeyName: !Ref KeyName
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

  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
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

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the inbound port
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: '80'
          ToPort: '80'
          SourceSecurityGroupOwnerId: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.OwnerAlias
          SourceSecurityGroupName: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.GroupName
        - IpProtocol: tcp
          FromPort: '22'
          ToPort: '22'
          CidrIp: !Ref SSHLocation

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

Description: 'AWS CloudFormation Sample Template ELBWithLockedDownAutoScaledInstances: Create a load balanced, Auto Scaled sample website where the instances are locked down to only accept traffic from the load balancer. This example creates an Auto Scaling group behind a load balancer with a simple health check. The web site is available on port 80, however, the instances can be configured to listen on any port (8888 by default). **WARNING** This template creates one or more Amazon EC2 instances and an Elastic Load Balancer. You will be billed for the AWS resources used if you create a stack from this template.'

Metadata:
  License: Apache-2.0

Parameters:
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

Resources:
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
      MinSize: '2'
      MaxSize: '2'
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
      ImageId: !Ref LatestAmiId
      SecurityGroups:
        - !Ref InstanceSecurityGroup
      InstanceType: !Ref InstanceType
      KeyName: !Ref KeyName
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

  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
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

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the inbound port
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: '80'
          ToPort: '80'
          SourceSecurityGroupOwnerId: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.OwnerAlias
          SourceSecurityGroupName: !GetAtt ElasticLoadBalancer.SourceSecurityGroup.GroupName
        - IpProtocol: tcp
          FromPort: '22'
          ToPort: '22'
          CidrIp: !Ref SSHLocation

Outputs:
  URL:
    Description: URL of the website
    Value: !Join
      - ''
      - - http://
        - !GetAtt ElasticLoadBalancer.DNSName

`````
