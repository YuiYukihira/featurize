# Changelog

## 0.1.0 (2024-04-25)


### Features

* A whole bunch of refactors and fixes lol ([68aa325](https://github.com/YuiYukihira/featurize/commit/68aa325743b0650e6652e0fae840246dad28a24c))
* Add Anti-CSRF token service ([d464c0c](https://github.com/YuiYukihira/featurize/commit/d464c0c64d515666ea4820845f8b5f8f37a9eb93))
* Add basis for auth server ([fb305ec](https://github.com/YuiYukihira/featurize/commit/fb305ec321a17184af96dd499d1c2376ad36c0ce))
* Add basis for settings menu ([#40](https://github.com/YuiYukihira/featurize/issues/40)) ([a78bb12](https://github.com/YuiYukihira/featurize/commit/a78bb120cd0d35cb6914882cb921a1ffc237ec65))
* Add consent page ([50b3b6d](https://github.com/YuiYukihira/featurize/commit/50b3b6d7fdcba58b341b06122aa3663e9ce418a2))
* Add custom user identity schema ([972422c](https://github.com/YuiYukihira/featurize/commit/972422cfbd92eaefa4a5b176ea9016a7fb96b306)), closes [#36](https://github.com/YuiYukihira/featurize/issues/36)
* Add custom user identity schema ([#60](https://github.com/YuiYukihira/featurize/issues/60)) ([906ba8c](https://github.com/YuiYukihira/featurize/commit/906ba8c9045e5b8c611e1ae175e93d0e79bdb0e8)), closes [#36](https://github.com/YuiYukihira/featurize/issues/36)
* Add dark mode ([bd59d95](https://github.com/YuiYukihira/featurize/commit/bd59d95b10dd4b28564e492e9b14a184563a4035))
* Add featurize homepage ([de2eef7](https://github.com/YuiYukihira/featurize/commit/de2eef7e63fc22f78754a028dd4039d80c8e9c45))
* Add method to accept OAuth2 consent request ([19b524a](https://github.com/YuiYukihira/featurize/commit/19b524a289f9205fd0594d369ea5b1df0625a5da))
* Add method to get OAuth2 consent request ([86d8f23](https://github.com/YuiYukihira/featurize/commit/86d8f2302622d0b12a7de329c1658cee4af1b1bb))
* Add method to reject OAuth2 consent request ([ad265d6](https://github.com/YuiYukihira/featurize/commit/ad265d6a03a979d9b514d30145a673ce1e26c167))
* Add minimal config for hydra ([8b4e6fa](https://github.com/YuiYukihira/featurize/commit/8b4e6fa367708f7317e98a55ba2aa428542c04e3))
* Add nicer UI for login/registration ([fdc14ae](https://github.com/YuiYukihira/featurize/commit/fdc14aea6b8bf6b1f70c11229e11a1a5bc4d7f1c))
* Add recovery page ([3be7621](https://github.com/YuiYukihira/featurize/commit/3be7621941d18667cc1c5169d06ec35eb0f7fa6b)), closes [#34](https://github.com/YuiYukihira/featurize/issues/34)
* Add sentry/tracing ([c5c026b](https://github.com/YuiYukihira/featurize/commit/c5c026bf9781de3a29b338569ee0c1e9cbc911fa))
* Get dev env ready ([59cbdaf](https://github.com/YuiYukihira/featurize/commit/59cbdaf7e66494786b343e09c5c9a592315d037b))
* Get sort of working on fly.io ([f2cd023](https://github.com/YuiYukihira/featurize/commit/f2cd02301a2d46d42f76205060dd240fa7db7643))
* Get user session via extractor ([de9df48](https://github.com/YuiYukihira/featurize/commit/de9df484266d45e0a8fb8767dc1b9a9ca7e1c60b))
* Make hydra use auth for login ([8d201a2](https://github.com/YuiYukihira/featurize/commit/8d201a26b71051c82195dd35626409a041453c27))
* Refresh expired login flows automatically ([79a4d1a](https://github.com/YuiYukihira/featurize/commit/79a4d1ae260162ea81d8c62f9256f9bebc4465eb)), closes [#28](https://github.com/YuiYukihira/featurize/issues/28)
* Use better templating inheritance ([aaa7851](https://github.com/YuiYukihira/featurize/commit/aaa78511907073ebcc62c75974225c2f224cfe2b))


### Bug Fixes

* Correct Cargo.lock location ([b30865d](https://github.com/YuiYukihira/featurize/commit/b30865d4e85937929b3ecf05845607329a06d5b4))
* **deps:** update rust crate reqwest to 0.12.3 ([0cf91a5](https://github.com/YuiYukihira/featurize/commit/0cf91a52225d5c8fb7dd955e606dc14cd6b487d3))
* **deps:** update rust crate reqwest to 0.12.3 ([#19](https://github.com/YuiYukihira/featurize/issues/19)) ([0f3e249](https://github.com/YuiYukihira/featurize/commit/0f3e2497e56a67b68550ff1c0ff7c356c3caa08a))
* **deps:** update rust crate reqwest to 0.12.4 ([134d01f](https://github.com/YuiYukihira/featurize/commit/134d01fcf69cde178648876d007072c61e28e20b))
* **deps:** update rust crate reqwest to 0.12.4 ([#66](https://github.com/YuiYukihira/featurize/issues/66)) ([b49c2ef](https://github.com/YuiYukihira/featurize/commit/b49c2eff7a95d30f11bc0e852f040fbcea58c4b4))
* **deps:** update rust crate sentry-tracing to 0.32.3 ([6810603](https://github.com/YuiYukihira/featurize/commit/68106035f5289edb9d361e8ab8658bee9814f023))
* **deps:** update rust crate sentry-tracing to 0.32.3 ([#61](https://github.com/YuiYukihira/featurize/issues/61)) ([51b7184](https://github.com/YuiYukihira/featurize/commit/51b71848014a8ef0adf5bd64effebefffbc5ccca))
* **deps:** update rust crate serde to 1.0.198 ([7b811a0](https://github.com/YuiYukihira/featurize/commit/7b811a057a8b5f6d56501a8c988453f3489823b5))
* **deps:** update rust crate serde to 1.0.198 ([#64](https://github.com/YuiYukihira/featurize/issues/64)) ([f763b8d](https://github.com/YuiYukihira/featurize/commit/f763b8d2adee1fd1536f2e1329448fc4853942fc))
* **deps:** update rust crate serde_json to 1.0.115 ([ddbdbac](https://github.com/YuiYukihira/featurize/commit/ddbdbaccc69b63066c84a36c48cb38087fe0be7d))
* **deps:** update rust crate serde_json to 1.0.115 ([#20](https://github.com/YuiYukihira/featurize/issues/20)) ([9b60c26](https://github.com/YuiYukihira/featurize/commit/9b60c2687315813dd11f04620d61ec4ed0a75a49))
* **deps:** update rust crate serde_json to 1.0.116 ([d0ddee7](https://github.com/YuiYukihira/featurize/commit/d0ddee7793334c6cdecf95662277790287daa7ef))
* **deps:** update rust crate serde_json to 1.0.116 ([#53](https://github.com/YuiYukihira/featurize/issues/53)) ([50ef20c](https://github.com/YuiYukihira/featurize/commit/50ef20c13ef2036edcc6313492e35d23caf04b36))
* **deps:** update rust crate thiserror to 1.0.59 ([a2384c2](https://github.com/YuiYukihira/featurize/commit/a2384c29659dc40494c08dcd21bdc3f5692d37bf))
* **deps:** update rust crate thiserror to 1.0.59 ([#67](https://github.com/YuiYukihira/featurize/issues/67)) ([8e295b5](https://github.com/YuiYukihira/featurize/commit/8e295b5a52481039757cc90419f0493913e7fdb6))
* **deps:** update sentry-rust monorepo to 0.32.3 ([7b7aa79](https://github.com/YuiYukihira/featurize/commit/7b7aa79f0075f1f35aba876663e582a7af2ebd63))
* **deps:** update sentry-rust monorepo to 0.32.3 ([#63](https://github.com/YuiYukihira/featurize/issues/63)) ([ee5192e](https://github.com/YuiYukihira/featurize/commit/ee5192e0106711455065dc24b2ccad1053b575bd))
* Make pages have html Content-Type ([1e16ef0](https://github.com/YuiYukihira/featurize/commit/1e16ef0fd7e756e8ad89f65bdf5e4097faf54b12))
* Submit buttons padding and alignment ([3e65fd0](https://github.com/YuiYukihira/featurize/commit/3e65fd01daf4861c01c445862506922b976093c0))
* Use labels with inputs ([b424887](https://github.com/YuiYukihira/featurize/commit/b4248872a5ee742968f3cdb24f6d85f483a55a4e))
