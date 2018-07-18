
### Linux TTS Service using Amazon Polly


There are 3 components (binaries) in this project:
  1. `tts-service` - a dbus daemon that listens for text to speech commands.
  2. `say` - a tool line tool that sends to `tts-service` the arguments or the piped input.
  3. `speak-selection` a tool that sends to `tts-service` the content of the X11 selection.
      You can bind this one to a key shortcut in order to make reading more pleasant.

#### Requirements

   AWS account in order to access the Polly service and an access key that can be used to use it.

#### Configuration

  If you have already an usable access key in your `.aws/credentials` it would be picked up.
  The default region is `eu-west-1` you might want to change it to something closer to you.
  The default voice is `Salli` in order to change you'll need to use `.config/tts.toml`
  configuration method.

  Using `.config/tts.toml` you can override the credentials and region from `.aws/credentials`

  ```toml
  voice = "<one of the voices from the list excluding the language>"
  speack_rate = "x-slow|slow|medium|fast|x-fast"
  #more on under the rate attribute https://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#prosody-tag
  aws_access_key = "a valid aws access key id"
  aws_secret_key = "a valid aws secret key"
  aws_region = "the aws region to be used"
  ```
  All the fields are optional.

#### Running

  You can run it as a systemd service but it needs to be under an X11 session or you can start the
  `tts-service` using your X11 desktop environment auto start support.
  On ubuntu and debian after installing the deb you can run `systemclt enable --user tts` and `systemctl start --user tts`
  after you `enable-linger` using `loginctl` for the user that you use in X11.

#### Voices

Filiz - tr-TR
Astrid - sv-SE
Tatyana - ru-RU
Maxim - ru-RU
Carmen - ro-RO
Inês - pt-PT
Cristiano - pt-PT
Vitória - pt-BR
Ricardo - pt-BR
Maja - pl-PL
Jan - pl-PL
Jacek - pl-PL
Ewa - pl-PL
Ruben - nl-NL
Lotte - nl-NL
Liv - nb-NO
Seoyeon - ko-KR
Takumi - ja-JP
Mizuki - ja-JP
Giorgio - it-IT
Carla - it-IT
Karl - is-IS
Dóra - is-IS
Mathieu - fr-FR
Léa - fr-FR
Céline - fr-FR
Chantal - fr-CA
Penélope - es-US
Miguel - es-US
Enrique - es-ES
Conchita - es-ES
Geraint - en-GB-WLS
Salli - en-US
Matthew - en-US
Kimberly - en-US
Kendra - en-US
Justin - en-US
Joey - en-US
Joanna - en-US
Ivy - en-US
Raveena - en-IN
Aditi - en-IN
Emma - en-GB
Brian - en-GB
Amy - en-GB
Russell - en-AU
Nicole - en-AU
Vicki - de-DE
Marlene - de-DE
Hans - de-DE
Naja - da-DK
Mads - da-DK
