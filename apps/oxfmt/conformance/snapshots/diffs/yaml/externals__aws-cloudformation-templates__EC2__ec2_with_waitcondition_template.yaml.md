# externals/aws-cloudformation-templates/EC2/ec2_with_waitcondition_template.yaml

## Option 2

`````json
{"printWidth":100,"tabWidth":4,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -190,9 +190,9 @@
                       - !Ref KWOSWaitHandle
                       - |
                           '
                       - |2
-                exit 1
+                          exit 1
                       - |
                           }
                       - |
                           # Install the basic system configuration
@@ -201,9 +201,9 @@
                       - " -r KWOSInstance "
                       - "         --region "
                       - !Ref AWS::Region
                       - |2
-               || error_exit 'Failed to run cfn-init'
+                         || error_exit 'Failed to run cfn-init'
                       - |
                           # All done so signal success
                       - /usr/local/bin/cfn-signal -e 0 -r "KWOS setup complete" '
                       - !Ref KWOSWaitHandle

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: Create a variable number of EC2 instance resources.

Parameters:
    KeyName:
        Description: Name of an existing EC2 KeyPair to enable SSH access to the instance
        Type: AWS::EC2::KeyPair::KeyName
        Default: slinger_testing
        ConstraintDescription: must be the name of an existing EC2 KeyPair.

    InstanceName:
        Description: Name of EC2 instance
        Type: String
        ConstraintDescription: must be a valid EC2 instance string name.

    InstanceType:
        Description: Basic EC2 instance type
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
        Default: c4.2xlarge
        ConstraintDescription: must be a valid EC2 instance type.

    ImageId:
        Description: Basic instance ami
        Type: AWS::EC2::Image::Id

    VpcId:
        Description: VpcId of your existing Virtual Private Cloud (VPC)
        Type: String

    SubnetId:
        Description: SubnetId of an existing subnet in your Virtual Private Cloud (VPC)
        Type: String

    SSHLocation:
        Description: " The IP address range that can be used to SSH to the EC2 instances"
        Type: String
        Default: 0.0.0.0/0
        MinLength: "9"
        MaxLength: "18"
        AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
        ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

    BudgetCode:
        Description: " Budget code to save money"
        Type: String
        Default: A019517
        ConstraintDescription: must be a valid budget code.

    LaunchPlatform:
        Description: " Mark current platform"
        Type: String
        Default: bitstorm_dev
        ConstraintDescription:
            must be a valid platform like bitstorm_qc bitstorm_dev bitstorm_staggin bitstorm_live.

    LaunchUser:
        Description: " Mark current tester"
        Type: String
        Default: "null"
        ConstraintDescription: must be a valid and existing tester.

    TestID:
        Description: " Mark current testcase"
        Type: String
        ConstraintDescription: must be a valid and existing testcase id.

    TestTarget:
        Description: " Mark current test target"
        Type: String
        ConstraintDescription: must be a valid and existing test target name.

    AgentID:
        Description: " Mark current agent"
        Type: String

    IsMaster:
        Description: Mark master agent
        Type: String
        Default: "False"

    MasterID:
        Description: Mark master ID
        Type: String
        Default: "null"

Mappings: {}

Resources:
    KWOSInstance:
        Type: AWS::EC2::Instance
        Metadata:
            AWS::CloudFormation::Init: {}
        Properties:
            ImageId: !Ref ImageId
            InstanceType: !Ref InstanceType
            SubnetId: !Ref SubnetId
            SecurityGroupIds:
                - !Ref KWOSSecurityGroup
            KeyName: !Ref KeyName
            Tags:
                - Key: LaunchPlatform
                  Value: !Ref LaunchPlatform
                - Key: LaunchUser
                  Value: !Ref LaunchUser
                - Key: TestID
                  Value: !Ref TestID
                - Key: Name
                  Value: !Ref InstanceName
                - Key: BudgetCode
                  Value: !Ref BudgetCode
                - Key: TestTarget
                  Value: !Ref TestTarget
                - Key: AgentID
                  Value: !Ref AgentID
                - Key: IsMaster
                  Value: !Ref IsMaster
                - Key: MasterID
                  Value: !Ref MasterID
            Monitoring: false
            UserData: !Base64
                Fn::Join:
                    - ""
                    - - |
                          #!/bin/bash
                      - |
                          apt-get -y install python-pip
                      - |
                          pip install https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-latest.tar.gz
                      - |
                          # Helper function
                      - |
                          function error_exit
                      - |
                          {
                      - '  /usr/local/bin/cfn-signal -e 1 -r "$1" '''
                      - !Ref KWOSWaitHandle
                      - |
                          '
                      - |2
                          exit 1
                      - |
                          }
                      - |
                          # Install the basic system configuration
                      - "/usr/local/bin/cfn-init -s "
                      - !Ref AWS::StackId
                      - " -r KWOSInstance "
                      - "         --region "
                      - !Ref AWS::Region
                      - |2
                         || error_exit 'Failed to run cfn-init'
                      - |
                          # All done so signal success
                      - /usr/local/bin/cfn-signal -e 0 -r "KWOS setup complete" '
                      - !Ref KWOSWaitHandle
                      - |
                          '

    KWOSWaitHandle:
        Type: AWS::CloudFormation::WaitConditionHandle

    KWOSWaitCondition:
        Type: AWS::CloudFormation::WaitCondition
        Properties:
            Handle: !Ref KWOSWaitHandle
            Timeout: "300"

    KWOSSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            VpcId: !Ref VpcId
            GroupDescription: Enable HTTP access via port 80/22/443 and ICMP access via port *
            SecurityGroupIngress:
                - IpProtocol: tcp
                  FromPort: "80"
                  ToPort: "80"
                  CidrIp: 0.0.0.0/0
                - IpProtocol: tcp
                  FromPort: "8888"
                  ToPort: "8888"
                  CidrIp: 0.0.0.0/0
                - IpProtocol: tcp
                  FromPort: "443"
                  ToPort: "443"
                  CidrIp: 0.0.0.0/0
                - IpProtocol: icmp
                  FromPort: "-1"
                  ToPort: "-1"
                  CidrIp: 0.0.0.0/0
                - IpProtocol: tcp
                  FromPort: "22"
                  ToPort: "22"
                  CidrIp: !Ref SSHLocation

Outputs:
    WebsiteURL:
        Description: URL for newly created KWOS deploy stack
        Value: !Join
            - ""
            - - http://
              - !GetAtt KWOSInstance.PublicDnsName

    InstanceId:
        Description: Instance Id of newly created instance
        Value: !Ref KWOSInstance

`````

### Expected (prettier)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: Create a variable number of EC2 instance resources.

Parameters:
    KeyName:
        Description: Name of an existing EC2 KeyPair to enable SSH access to the instance
        Type: AWS::EC2::KeyPair::KeyName
        Default: slinger_testing
        ConstraintDescription: must be the name of an existing EC2 KeyPair.

    InstanceName:
        Description: Name of EC2 instance
        Type: String
        ConstraintDescription: must be a valid EC2 instance string name.

    InstanceType:
        Description: Basic EC2 instance type
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
        Default: c4.2xlarge
        ConstraintDescription: must be a valid EC2 instance type.

    ImageId:
        Description: Basic instance ami
        Type: AWS::EC2::Image::Id

    VpcId:
        Description: VpcId of your existing Virtual Private Cloud (VPC)
        Type: String

    SubnetId:
        Description: SubnetId of an existing subnet in your Virtual Private Cloud (VPC)
        Type: String

    SSHLocation:
        Description: " The IP address range that can be used to SSH to the EC2 instances"
        Type: String
        Default: 0.0.0.0/0
        MinLength: "9"
        MaxLength: "18"
        AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
        ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

    BudgetCode:
        Description: " Budget code to save money"
        Type: String
        Default: A019517
        ConstraintDescription: must be a valid budget code.

    LaunchPlatform:
        Description: " Mark current platform"
        Type: String
        Default: bitstorm_dev
        ConstraintDescription:
            must be a valid platform like bitstorm_qc bitstorm_dev bitstorm_staggin bitstorm_live.

    LaunchUser:
        Description: " Mark current tester"
        Type: String
        Default: "null"
        ConstraintDescription: must be a valid and existing tester.

    TestID:
        Description: " Mark current testcase"
        Type: String
        ConstraintDescription: must be a valid and existing testcase id.

    TestTarget:
        Description: " Mark current test target"
        Type: String
        ConstraintDescription: must be a valid and existing test target name.

    AgentID:
        Description: " Mark current agent"
        Type: String

    IsMaster:
        Description: Mark master agent
        Type: String
        Default: "False"

    MasterID:
        Description: Mark master ID
        Type: String
        Default: "null"

Mappings: {}

Resources:
    KWOSInstance:
        Type: AWS::EC2::Instance
        Metadata:
            AWS::CloudFormation::Init: {}
        Properties:
            ImageId: !Ref ImageId
            InstanceType: !Ref InstanceType
            SubnetId: !Ref SubnetId
            SecurityGroupIds:
                - !Ref KWOSSecurityGroup
            KeyName: !Ref KeyName
            Tags:
                - Key: LaunchPlatform
                  Value: !Ref LaunchPlatform
                - Key: LaunchUser
                  Value: !Ref LaunchUser
                - Key: TestID
                  Value: !Ref TestID
                - Key: Name
                  Value: !Ref InstanceName
                - Key: BudgetCode
                  Value: !Ref BudgetCode
                - Key: TestTarget
                  Value: !Ref TestTarget
                - Key: AgentID
                  Value: !Ref AgentID
                - Key: IsMaster
                  Value: !Ref IsMaster
                - Key: MasterID
                  Value: !Ref MasterID
            Monitoring: false
            UserData: !Base64
                Fn::Join:
                    - ""
                    - - |
                          #!/bin/bash
                      - |
                          apt-get -y install python-pip
                      - |
                          pip install https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-latest.tar.gz
                      - |
                          # Helper function
                      - |
                          function error_exit
                      - |
                          {
                      - '  /usr/local/bin/cfn-signal -e 1 -r "$1" '''
                      - !Ref KWOSWaitHandle
                      - |
                          '
                      - |2
                exit 1
                      - |
                          }
                      - |
                          # Install the basic system configuration
                      - "/usr/local/bin/cfn-init -s "
                      - !Ref AWS::StackId
                      - " -r KWOSInstance "
                      - "         --region "
                      - !Ref AWS::Region
                      - |2
               || error_exit 'Failed to run cfn-init'
                      - |
                          # All done so signal success
                      - /usr/local/bin/cfn-signal -e 0 -r "KWOS setup complete" '
                      - !Ref KWOSWaitHandle
                      - |
                          '

    KWOSWaitHandle:
        Type: AWS::CloudFormation::WaitConditionHandle

    KWOSWaitCondition:
        Type: AWS::CloudFormation::WaitCondition
        Properties:
            Handle: !Ref KWOSWaitHandle
            Timeout: "300"

    KWOSSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            VpcId: !Ref VpcId
            GroupDescription: Enable HTTP access via port 80/22/443 and ICMP access via port *
            SecurityGroupIngress:
                - IpProtocol: tcp
                  FromPort: "80"
                  ToPort: "80"
                  CidrIp: 0.0.0.0/0
                - IpProtocol: tcp
                  FromPort: "8888"
                  ToPort: "8888"
                  CidrIp: 0.0.0.0/0
                - IpProtocol: tcp
                  FromPort: "443"
                  ToPort: "443"
                  CidrIp: 0.0.0.0/0
                - IpProtocol: icmp
                  FromPort: "-1"
                  ToPort: "-1"
                  CidrIp: 0.0.0.0/0
                - IpProtocol: tcp
                  FromPort: "22"
                  ToPort: "22"
                  CidrIp: !Ref SSHLocation

Outputs:
    WebsiteURL:
        Description: URL for newly created KWOS deploy stack
        Value: !Join
            - ""
            - - http://
              - !GetAtt KWOSInstance.PublicDnsName

    InstanceId:
        Description: Instance Id of newly created instance
        Value: !Ref KWOSInstance

`````
