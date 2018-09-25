use lewton;
use rusoto_polly;

error_chain!{
  foreign_links {
    VorbisError(lewton::VorbisError);
    SynthesizeSpeechError(rusoto_polly::SynthesizeSpeechError);
  }
}
