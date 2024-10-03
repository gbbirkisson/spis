# Changelog

## [0.8.1](https://github.com/gbbirkisson/spis/compare/v0.8.0...v0.8.1) (2024-10-03)


### Bug Fixes

* **deps:** update dependency rust to v1.80.1 ([#273](https://github.com/gbbirkisson/spis/issues/273)) ([b77153e](https://github.com/gbbirkisson/spis/commit/b77153e222c280d77ce2016113f1cfe87bc1f9de))
* **deps:** update dependency rust to v1.81.0 ([#274](https://github.com/gbbirkisson/spis/issues/274)) ([f69268b](https://github.com/gbbirkisson/spis/commit/f69268b0eb42b444b56063175d189a84c63a4240))
* **deps:** update nginx docker tag to v1.27.2 ([#287](https://github.com/gbbirkisson/spis/issues/287)) ([923fa96](https://github.com/gbbirkisson/spis/commit/923fa964473257fd2bc00509c456f88fc332d575))
* **deps:** update rust crate clap to v4.5.17 ([#275](https://github.com/gbbirkisson/spis/issues/275)) ([5279414](https://github.com/gbbirkisson/spis/commit/5279414e43bfd342ab305f6c869142b6469eb225))
* **deps:** update rust crate clap to v4.5.18 ([#280](https://github.com/gbbirkisson/spis/issues/280)) ([f4959b2](https://github.com/gbbirkisson/spis/commit/f4959b2fbb22da6a041f273b45dc6bc6f5cb3d57))
* **deps:** update rust crate clap to v4.5.19 ([#286](https://github.com/gbbirkisson/spis/issues/286)) ([983dae1](https://github.com/gbbirkisson/spis/commit/983dae17194dfbfbe90f02ee75b4b2e024bcd599))
* **deps:** update rust crate reqwest to v0.12.8 ([#285](https://github.com/gbbirkisson/spis/issues/285)) ([7a5b0ad](https://github.com/gbbirkisson/spis/commit/7a5b0ad1ff98c0b7d39d483393f2bc3dbbf66c56))
* **deps:** update rust crate serde to v1.0.209 ([#271](https://github.com/gbbirkisson/spis/issues/271)) ([1b54ea8](https://github.com/gbbirkisson/spis/commit/1b54ea889ba101b08a7b29223f00f7db7d62a421))
* **deps:** update rust crate serde to v1.0.210 ([#276](https://github.com/gbbirkisson/spis/issues/276)) ([05ed6d1](https://github.com/gbbirkisson/spis/commit/05ed6d118705e21eef9ef7734ad6883d2e791c05))
* **deps:** update rust crate tempfile to v3.13.0 ([#283](https://github.com/gbbirkisson/spis/issues/283)) ([5197a87](https://github.com/gbbirkisson/spis/commit/5197a87c752363479a40887ab9df420c70e850b6))
* **deps:** update rust crate thiserror to v1.0.64 ([#281](https://github.com/gbbirkisson/spis/issues/281)) ([fa0ca0b](https://github.com/gbbirkisson/spis/commit/fa0ca0b76d9fdff85053e9af84da1b5109cbcf49))
* **deps:** update rust crate tokio to v1.40.0 ([#278](https://github.com/gbbirkisson/spis/issues/278)) ([a78495b](https://github.com/gbbirkisson/spis/commit/a78495b322d6ae841971e61c73820acbba1e0587))
* **deps:** update rust crate tracing-actix-web to v0.7.12 ([#277](https://github.com/gbbirkisson/spis/issues/277)) ([073da9c](https://github.com/gbbirkisson/spis/commit/073da9c61cc7da960b0738b7b42cfbbddb7bdf60))
* **deps:** update rust crate tracing-actix-web to v0.7.13 ([#282](https://github.com/gbbirkisson/spis/issues/282)) ([94b27a2](https://github.com/gbbirkisson/spis/commit/94b27a2f9b958c7f9771a061fe561da6caf9ee97))

## [0.8.0](https://github.com/gbbirkisson/spis/compare/v0.7.0...v0.8.0) (2024-08-22)


### Features

* **processing:** follow symlinks by default ([#267](https://github.com/gbbirkisson/spis/issues/267)) ([963f9c5](https://github.com/gbbirkisson/spis/commit/963f9c5fba4ebfbb96dc3e476d50d82bbf2bad45))
* use creation time if exif data is missing ([#270](https://github.com/gbbirkisson/spis/issues/270)) ([d6fa5a5](https://github.com/gbbirkisson/spis/commit/d6fa5a56e47b0b9fad2498c291c4a29d8b27f5b2))


### Bug Fixes

* bump page size to 100 ([#269](https://github.com/gbbirkisson/spis/issues/269)) ([09f8381](https://github.com/gbbirkisson/spis/commit/09f83813ed3493603acec45ca771952fd893bcae))
* **templates:** remove whitespaces ([#266](https://github.com/gbbirkisson/spis/issues/266)) ([c0bd908](https://github.com/gbbirkisson/spis/commit/c0bd9087ff248e42d612a5ec71f4f16bbd347fa8))

## [0.7.0](https://github.com/gbbirkisson/spis/compare/v0.6.0...v0.7.0) (2024-08-22)


### Features

* add configuration file templating ([#263](https://github.com/gbbirkisson/spis/issues/263)) ([81dc9fc](https://github.com/gbbirkisson/spis/commit/81dc9fcbed3fd59b9a36fdf5c0f3d2b6db162617))
* Rewrite using askama and htmx ([#179](https://github.com/gbbirkisson/spis/issues/179)) ([c8aa488](https://github.com/gbbirkisson/spis/commit/c8aa4883d32898c6690125c8691deac06a7e253d))


### Bug Fixes

* Add extra label to release-please PRs ([85952de](https://github.com/gbbirkisson/spis/commit/85952dedcde64aad944f8e0c9b2deffb3641fea4))
* Add pre-commit hook for conventional commits ([fdc95e8](https://github.com/gbbirkisson/spis/commit/fdc95e86853bf4e67156a6230d52fff2ddb49182))
* Add release-please ([94ccc2e](https://github.com/gbbirkisson/spis/commit/94ccc2ec0eec6bc8fec59ec6fd87a40c85859174))
* **CI:** continue on failed codecov uploads ([#243](https://github.com/gbbirkisson/spis/issues/243)) ([9f71b46](https://github.com/gbbirkisson/spis/commit/9f71b461644fe1939ca73fb6a28749376f4117e9))
* **CI:** explicitly set db url ([#248](https://github.com/gbbirkisson/spis/issues/248)) ([8fcfc2f](https://github.com/gbbirkisson/spis/commit/8fcfc2f1d1cdf04b0cb042afd025badcad343511))
* **CI:** fix release ([#245](https://github.com/gbbirkisson/spis/issues/245)) ([024a35e](https://github.com/gbbirkisson/spis/commit/024a35eb41ffacd9ad0ca902a031d84857fb0277))
* **CI:** rework cross compilation ([#258](https://github.com/gbbirkisson/spis/issues/258)) ([6c628d0](https://github.com/gbbirkisson/spis/commit/6c628d0a4bce4bf71c1d385c8d70d6ade1574425))
* **deps:** remove derive_more ([#239](https://github.com/gbbirkisson/spis/issues/239)) ([3ed34a3](https://github.com/gbbirkisson/spis/commit/3ed34a3050e1aa5b6a648e812a610f83d5d73281))
* **deps:** update actions/download-artifact action to v4 ([#165](https://github.com/gbbirkisson/spis/issues/165)) ([f561cb0](https://github.com/gbbirkisson/spis/commit/f561cb07568e71647126a2b4ace33cae5409fd9e))
* **deps:** update actions/stale action to v9 ([#163](https://github.com/gbbirkisson/spis/issues/163)) ([ba739a0](https://github.com/gbbirkisson/spis/commit/ba739a04d72f75e10cd9557b10fba301e3b4d9d4))
* **deps:** update actions/upload-artifact action to v4 ([#164](https://github.com/gbbirkisson/spis/issues/164)) ([9424bd0](https://github.com/gbbirkisson/spis/commit/9424bd028e880bdec41af6828ab97323c0e1c1a2))
* **deps:** update cargo-tarpaulin to 0.31.0 ([cf6ffd2](https://github.com/gbbirkisson/spis/commit/cf6ffd2226e25b0f1078acb6643f4b3b18cbbc17))
* **deps:** update codecov/codecov-action action to v4 ([#177](https://github.com/gbbirkisson/spis/issues/177)) ([f649d25](https://github.com/gbbirkisson/spis/commit/f649d2537dafe279d754a9817f662d15e00b83b0))
* **deps:** update dependency rust to v1.74.0 ([#150](https://github.com/gbbirkisson/spis/issues/150)) ([3ee6487](https://github.com/gbbirkisson/spis/commit/3ee6487b51f1474eec44f6fe8472ee3f21dccd7d))
* **deps:** update dependency rust to v1.74.1 ([#166](https://github.com/gbbirkisson/spis/issues/166)) ([4ac7933](https://github.com/gbbirkisson/spis/commit/4ac7933ba595b2de69209c94472eead1018c721d))
* **deps:** update dependency rust to v1.75.0 ([#167](https://github.com/gbbirkisson/spis/issues/167)) ([4cbab5d](https://github.com/gbbirkisson/spis/commit/4cbab5dc5a749fc9da288d9bf635c225ab027445))
* **deps:** update dependency rust to v1.80.0 ([#213](https://github.com/gbbirkisson/spis/issues/213)) ([ca913eb](https://github.com/gbbirkisson/spis/commit/ca913eb5275d4b4364e3458997cc1cd30933ee54))
* **deps:** update docker/build-push-action action to v6 ([#215](https://github.com/gbbirkisson/spis/issues/215)) ([5c62809](https://github.com/gbbirkisson/spis/commit/5c6280946cfc6543f603adf0fe3a5f896f732113))
* **deps:** update htmx to 2.0.2 ([#240](https://github.com/gbbirkisson/spis/issues/240)) ([0c50252](https://github.com/gbbirkisson/spis/commit/0c502524afd06d3656b8183259d5d9fde184ba51))
* **deps:** update nginx docker tag to v1.25.3 ([#151](https://github.com/gbbirkisson/spis/issues/151)) ([bb830a6](https://github.com/gbbirkisson/spis/commit/bb830a6743b472ccea707beac8a640ab74cf3050))
* **deps:** update nginx docker tag to v1.27.0 ([#180](https://github.com/gbbirkisson/spis/issues/180)) ([f7de3e7](https://github.com/gbbirkisson/spis/commit/f7de3e737dcc0ec902a546d193575770815592bf))
* **deps:** update nginx docker tag to v1.27.1 ([#233](https://github.com/gbbirkisson/spis/issues/233)) ([bce807e](https://github.com/gbbirkisson/spis/commit/bce807ec2abb5087410f7a0e85d520a0583eec99))
* **deps:** update rust crate actix to v0.13.3 ([#192](https://github.com/gbbirkisson/spis/issues/192)) ([46ed80c](https://github.com/gbbirkisson/spis/commit/46ed80cc555a072e969b509950abba912eb5df90))
* **deps:** update rust crate actix to v0.13.5 ([#211](https://github.com/gbbirkisson/spis/issues/211)) ([a15d939](https://github.com/gbbirkisson/spis/commit/a15d93975ed34bede0b8984fd83ed3188c06cb40))
* **deps:** update rust crate actix-web to v4.6.0 ([#197](https://github.com/gbbirkisson/spis/issues/197)) ([2fcd405](https://github.com/gbbirkisson/spis/commit/2fcd405e886fbad7aa30418518c2e2a56072f054))
* **deps:** update rust crate actix-web to v4.8.0 ([#212](https://github.com/gbbirkisson/spis/issues/212)) ([4b32903](https://github.com/gbbirkisson/spis/commit/4b329032a43174c25b466cfe55fc8e4d4b759c2e))
* **deps:** update rust crate actix-web to v4.9.0 ([#232](https://github.com/gbbirkisson/spis/issues/232)) ([7cab30c](https://github.com/gbbirkisson/spis/commit/7cab30cf26f4231fb45481563cf80b5ae45868d2))
* **deps:** update rust crate actix-web-actors to v4.3.0 ([#198](https://github.com/gbbirkisson/spis/issues/198)) ([a14166e](https://github.com/gbbirkisson/spis/commit/a14166e0b682195ccca698e3a49910e98d1f8dfb))
* **deps:** update rust crate actix-web-actors to v4.3.1 ([#228](https://github.com/gbbirkisson/spis/issues/228)) ([5bb9456](https://github.com/gbbirkisson/spis/commit/5bb945633703ff42304e8300572dfc9995c62f19))
* **deps:** update rust crate async-cron-scheduler to v2 ([#168](https://github.com/gbbirkisson/spis/issues/168)) ([f7cb8c8](https://github.com/gbbirkisson/spis/commit/f7cb8c825438f63e061d66ade227d2ace152fee2))
* **deps:** update rust crate chrono to v0.4.38 ([#189](https://github.com/gbbirkisson/spis/issues/189)) ([7a58147](https://github.com/gbbirkisson/spis/commit/7a581477db14e414056c8a530040efe659ab9869))
* **deps:** update rust crate clap to v4.5.11 ([#208](https://github.com/gbbirkisson/spis/issues/208)) ([791aef6](https://github.com/gbbirkisson/spis/commit/791aef6b5c3a98d266e9d9dea2d9e3ca92c61dc1))
* **deps:** update rust crate clap to v4.5.13 ([#225](https://github.com/gbbirkisson/spis/issues/225)) ([4224c56](https://github.com/gbbirkisson/spis/commit/4224c566886a70c789cf2c228e91eb6bcd7dd68f))
* **deps:** update rust crate clap to v4.5.15 ([#231](https://github.com/gbbirkisson/spis/issues/231)) ([318002d](https://github.com/gbbirkisson/spis/commit/318002dec240c12670f8af654bb4219e715fe78c))
* **deps:** update rust crate clap to v4.5.16 ([#235](https://github.com/gbbirkisson/spis/issues/235)) ([281afb6](https://github.com/gbbirkisson/spis/commit/281afb662d5309f740e48d2ec4ed7e7259a21c51))
* **deps:** update rust crate clap to v4.5.4 ([#199](https://github.com/gbbirkisson/spis/issues/199)) ([6337765](https://github.com/gbbirkisson/spis/commit/6337765baace41e9dcb35478391574277072b315))
* **deps:** update rust crate color-eyre to v0.6.3 ([#190](https://github.com/gbbirkisson/spis/issues/190)) ([075bc6b](https://github.com/gbbirkisson/spis/commit/075bc6b99119e829aa99eeb4aaf7f7f75a4644e5))
* **deps:** update rust crate config to 0.14 ([#176](https://github.com/gbbirkisson/spis/issues/176)) ([9f2de68](https://github.com/gbbirkisson/spis/commit/9f2de68bba875b14fbcb6e13010a44d22ef7a1ec))
* **deps:** update rust crate derive_more to v0.99.18 ([#214](https://github.com/gbbirkisson/spis/issues/214)) ([9197f98](https://github.com/gbbirkisson/spis/commit/9197f98f5661110af7799c860455b7ba243f210e))
* **deps:** update rust crate image to 0.24.7 ([#147](https://github.com/gbbirkisson/spis/issues/147)) ([3813a7c](https://github.com/gbbirkisson/spis/commit/3813a7c1807d7780cd2720b810fa54cd8ef00a6a))
* **deps:** update rust crate image to 0.24.8 ([#171](https://github.com/gbbirkisson/spis/issues/171)) ([9c8f1b2](https://github.com/gbbirkisson/spis/commit/9c8f1b23074dad80f23fae611df806390eef665f))
* **deps:** update rust crate image to 0.25.0 ([#181](https://github.com/gbbirkisson/spis/issues/181)) ([24499a7](https://github.com/gbbirkisson/spis/commit/24499a7b35fceda401741b2570dad40dbb8d873b))
* **deps:** update rust crate image to v0.25.2 ([#220](https://github.com/gbbirkisson/spis/issues/220)) ([a11d13e](https://github.com/gbbirkisson/spis/commit/a11d13eba625fbf5b18ef96a27b02e793d5f9097))
* **deps:** update rust crate include_dir to v0.7.4 ([#216](https://github.com/gbbirkisson/spis/issues/216)) ([e15004d](https://github.com/gbbirkisson/spis/commit/e15004dc64127fc14dc3cf066584ecc4428aa13a))
* **deps:** update rust crate log to 0.4.20 ([#148](https://github.com/gbbirkisson/spis/issues/148)) ([84bbf10](https://github.com/gbbirkisson/spis/commit/84bbf1042cd9fcded981e16ca3052dcfbc742eb6))
* **deps:** update rust crate notify to 6.1.1 ([#154](https://github.com/gbbirkisson/spis/issues/154)) ([7e8b8ee](https://github.com/gbbirkisson/spis/commit/7e8b8ee925d1bc76d37739afe2b31e5d2c0397ac))
* **deps:** update rust crate rayon to v1.10.0 ([#200](https://github.com/gbbirkisson/spis/issues/200)) ([a068cda](https://github.com/gbbirkisson/spis/commit/a068cda2fe72ea03da1531ff2eee15ff418bc5d2))
* **deps:** update rust crate reqwest to 0.12 ([#185](https://github.com/gbbirkisson/spis/issues/185)) ([0db7e1c](https://github.com/gbbirkisson/spis/commit/0db7e1c868d93c220727cc5c33aff469c05eade3))
* **deps:** update rust crate reqwest to v0.12.5 ([#217](https://github.com/gbbirkisson/spis/issues/217)) ([231c573](https://github.com/gbbirkisson/spis/commit/231c57306978b6342fc87ff9daa028875389b506))
* **deps:** update rust crate reqwest to v0.12.6 ([#238](https://github.com/gbbirkisson/spis/issues/238)) ([dc07a6a](https://github.com/gbbirkisson/spis/commit/dc07a6a7976f94cb3895d15e786fd33ee8ac94f1))
* **deps:** update rust crate reqwest to v0.12.7 ([#246](https://github.com/gbbirkisson/spis/issues/246)) ([f0dcf67](https://github.com/gbbirkisson/spis/commit/f0dcf6703cb2dcf5b3bd89e12038728fc20db39a))
* **deps:** update rust crate serde to v1.0.203 ([#186](https://github.com/gbbirkisson/spis/issues/186)) ([e265734](https://github.com/gbbirkisson/spis/commit/e26573494513c4f6dc3e31c6eae9815935696ba0))
* **deps:** update rust crate serde to v1.0.204 ([#219](https://github.com/gbbirkisson/spis/issues/219)) ([8ae547e](https://github.com/gbbirkisson/spis/commit/8ae547eb1afe6b28558ae2297f428907d2cd5319))
* **deps:** update rust crate serde to v1.0.207 ([#230](https://github.com/gbbirkisson/spis/issues/230)) ([9e0c36f](https://github.com/gbbirkisson/spis/commit/9e0c36f34ab3c35302c9295465f53310e9e6d82b))
* **deps:** update rust crate serde to v1.0.208 ([#234](https://github.com/gbbirkisson/spis/issues/234)) ([0bc8484](https://github.com/gbbirkisson/spis/commit/0bc8484edd1971b8018e15dc3a2d56345721435f))
* **deps:** update rust crate sqlx to 0.7 ([#155](https://github.com/gbbirkisson/spis/issues/155)) ([01b79a7](https://github.com/gbbirkisson/spis/commit/01b79a7d1c2ec66c7e56f596c8fed9aefb230c09))
* **deps:** update rust crate sqlx to 0.8 ([#222](https://github.com/gbbirkisson/spis/issues/222)) ([6df03da](https://github.com/gbbirkisson/spis/commit/6df03daac4baad194900e88e4845deb5db0a8103))
* **deps:** update rust crate sqlx to v0.7.4 ([#193](https://github.com/gbbirkisson/spis/issues/193)) ([aa9c988](https://github.com/gbbirkisson/spis/commit/aa9c98821d6df3decb95e49b05d27111980cd5a9))
* **deps:** update rust crate tempfile to v3.10.1 ([#201](https://github.com/gbbirkisson/spis/issues/201)) ([9746ac2](https://github.com/gbbirkisson/spis/commit/9746ac2606714cd087296031ccd27fe3be9c3a1e))
* **deps:** update rust crate tempfile to v3.11.0 ([#226](https://github.com/gbbirkisson/spis/issues/226)) ([cf4a3f3](https://github.com/gbbirkisson/spis/commit/cf4a3f3b6e1a147506568329de91c5ea59d8eda9))
* **deps:** update rust crate tempfile to v3.12.0 ([#227](https://github.com/gbbirkisson/spis/issues/227)) ([6452bda](https://github.com/gbbirkisson/spis/commit/6452bda9db21b62436d50a95293ceb9f1f8f4275))
* **deps:** update rust crate thiserror to v1.0.61 ([#195](https://github.com/gbbirkisson/spis/issues/195)) ([df42e9b](https://github.com/gbbirkisson/spis/commit/df42e9b482c5f0334fac59f05fff0ba8a65fd5db))
* **deps:** update rust crate thiserror to v1.0.63 ([#221](https://github.com/gbbirkisson/spis/issues/221)) ([9aaf09d](https://github.com/gbbirkisson/spis/commit/9aaf09db279929c84b824cd30fce954c3e8b066a))
* **deps:** update rust crate tokio to v1.38.0 ([#202](https://github.com/gbbirkisson/spis/issues/202)) ([a28cab7](https://github.com/gbbirkisson/spis/commit/a28cab74d398f41466e64a5183bcd6a69d1e8cf7))
* **deps:** update rust crate tokio to v1.39.2 ([#223](https://github.com/gbbirkisson/spis/issues/223)) ([6c4a386](https://github.com/gbbirkisson/spis/commit/6c4a386e1db871bf5b541edaf1b51e010b5c678d))
* **deps:** update rust crate tokio to v1.39.3 ([#236](https://github.com/gbbirkisson/spis/issues/236)) ([08e1c2c](https://github.com/gbbirkisson/spis/commit/08e1c2ca259504c16cd6a5286cb39f0c62ef193d))
* **deps:** update rust crate tracing-actix-web to v0.7.10 ([#184](https://github.com/gbbirkisson/spis/issues/184)) ([6ec48a7](https://github.com/gbbirkisson/spis/commit/6ec48a7fec7f7446dd704e9f27e515b8811f94dc))
* **deps:** update rust crate tracing-actix-web to v0.7.11 ([#203](https://github.com/gbbirkisson/spis/issues/203)) ([c9f7d62](https://github.com/gbbirkisson/spis/commit/c9f7d6222506ef60f8fdca59fc2e04bae9d469e4))
* **deps:** update rust crate uuid to v1.10.0 ([#218](https://github.com/gbbirkisson/spis/issues/218)) ([9f19c87](https://github.com/gbbirkisson/spis/commit/9f19c87a698958b77c1106f0e4ff4f5549d77cce))
* **deps:** update rust crate uuid to v1.8.0 ([#204](https://github.com/gbbirkisson/spis/issues/204)) ([e1d06d8](https://github.com/gbbirkisson/spis/commit/e1d06d804c8d9b8553b77274981c7085abe49b1a))
* **deps:** update rust crate walkdir to v2.5.0 ([#205](https://github.com/gbbirkisson/spis/issues/205)) ([2091b84](https://github.com/gbbirkisson/spis/commit/2091b84c5951445abe389fa9eefed5a0969e23ea))
* **deps:** update rust crate which to 4.4 ([#156](https://github.com/gbbirkisson/spis/issues/156)) ([cefc6a5](https://github.com/gbbirkisson/spis/commit/cefc6a53103b27240b76e14754c04aee081fb082))
* **deps:** update rust crate which to v5 ([#157](https://github.com/gbbirkisson/spis/issues/157)) ([69521da](https://github.com/gbbirkisson/spis/commit/69521daf96a8ea2af75a6e7f9deed8c963ca836d))
* **deps:** update rust crate which to v6 ([#173](https://github.com/gbbirkisson/spis/issues/173)) ([d3e1523](https://github.com/gbbirkisson/spis/commit/d3e1523240a1e35c5e70352ff3e5f666573c596d))
* **deps:** update rust crate which to v6.0.1 ([#196](https://github.com/gbbirkisson/spis/issues/196)) ([4f932e6](https://github.com/gbbirkisson/spis/commit/4f932e63fdc9ccb0a47e69688760fcd62b5492ae))
* **deps:** update rust crate which to v6.0.2 ([#224](https://github.com/gbbirkisson/spis/issues/224)) ([b5f8163](https://github.com/gbbirkisson/spis/commit/b5f8163abe299e398e3c47e8f8b6c691ceed0168))
* **deps:** update rust crate which to v6.0.3 ([#237](https://github.com/gbbirkisson/spis/issues/237)) ([b7cd245](https://github.com/gbbirkisson/spis/commit/b7cd245527ee5d50b1356eb1af60f2459fbb9ba9))
* **deps:** update scribemd/docker-cache action to v0.5.0 ([#206](https://github.com/gbbirkisson/spis/issues/206)) ([2e6bb62](https://github.com/gbbirkisson/spis/commit/2e6bb62395f71406bcdc9c1c5fa12e8d936fc2ed))
* **deps:** update sqlx-cli to 0.8.0 ([#250](https://github.com/gbbirkisson/spis/issues/250)) ([650a150](https://github.com/gbbirkisson/spis/commit/650a150740ff399bbfa223ea730ebe6aa76efcf8))
* docker linting ([#241](https://github.com/gbbirkisson/spis/issues/241)) ([6d4227b](https://github.com/gbbirkisson/spis/commit/6d4227bf2e6aba169199d2a3e206d3193b0ea976))
* Docker tag ([ddd832b](https://github.com/gbbirkisson/spis/commit/ddd832bfd456309781d9761cfefaf710f4311603))
* Fix release-please tags ([9a8a7f3](https://github.com/gbbirkisson/spis/commit/9a8a7f346824d61faf3baef1600dfe0dfd0cdf43))
* **git:** update gitignore file ([#259](https://github.com/gbbirkisson/spis/issues/259)) ([1103c01](https://github.com/gbbirkisson/spis/commit/1103c0142bc39de5fcbdc463a7cfdc2a5bbdd055))
* **lint:** Clean up clippy linting ([#207](https://github.com/gbbirkisson/spis/issues/207)) ([3dc077c](https://github.com/gbbirkisson/spis/commit/3dc077c2cdee673e3687dc10fd69602b7bde5ba3))
* move templates around ([#262](https://github.com/gbbirkisson/spis/issues/262)) ([82cdb0d](https://github.com/gbbirkisson/spis/commit/82cdb0db271a09589235b79be02ec937a7de6741))
* overhaul configuration to only use clap ([#261](https://github.com/gbbirkisson/spis/issues/261)) ([2b41317](https://github.com/gbbirkisson/spis/commit/2b41317d8a53596f9dcb478671ad846d7cdbf559))
* **release:** remove component tag in release ([#252](https://github.com/gbbirkisson/spis/issues/252)) ([b526c60](https://github.com/gbbirkisson/spis/commit/b526c60c2f618e38e49eadd9fad76c3168b39970))
* remove audit config ([#260](https://github.com/gbbirkisson/spis/issues/260)) ([6e6f4d7](https://github.com/gbbirkisson/spis/commit/6e6f4d7d5e6f46f48feb78e655c0c16fa21fe8c7))
* Remove dependabot ([29fbc42](https://github.com/gbbirkisson/spis/commit/29fbc42722cc3570ad7051dcb730113eed40ae2c))
* Remove validate step ([047032a](https://github.com/gbbirkisson/spis/commit/047032a451c0ca46a77398aeca4878bc34477873))
* Run CI on toolchain change ([dfa3db2](https://github.com/gbbirkisson/spis/commit/dfa3db2de3b80af5114c39bb6d74b66163e05cd1))
* Update audit exceptions ([7f91064](https://github.com/gbbirkisson/spis/commit/7f91064385b0cdce9e4fab99cc9b8a02b95e1583))
* Update cargo configuration ([4039028](https://github.com/gbbirkisson/spis/commit/4039028ee68f17fc4a3fd25cf078da262f756346))
* Update renovate configuration ([6975373](https://github.com/gbbirkisson/spis/commit/6975373ac5ba408408890b623b92e683d0a4996a))
* Use font awesome instead of svg ([#161](https://github.com/gbbirkisson/spis/issues/161)) ([644e590](https://github.com/gbbirkisson/spis/commit/644e590bb4b7e014714ee4a0971a2568472aa3d0))


### Miscellaneous Chores

* release 0.6.0 ([6f5f42f](https://github.com/gbbirkisson/spis/commit/6f5f42fbe3226911d87e7903b8745ce80cb11ddb))

## [0.6.0](https://github.com/gbbirkisson/spis/compare/spis-v0.5.7...spis-v0.6.0) (2024-01-13)


### Bug Fixes

* **deps:** update actions/download-artifact action to v4 ([#165](https://github.com/gbbirkisson/spis/issues/165)) ([f561cb0](https://github.com/gbbirkisson/spis/commit/f561cb07568e71647126a2b4ace33cae5409fd9e))
* **deps:** update actions/stale action to v9 ([#163](https://github.com/gbbirkisson/spis/issues/163)) ([ba739a0](https://github.com/gbbirkisson/spis/commit/ba739a04d72f75e10cd9557b10fba301e3b4d9d4))
* **deps:** update actions/upload-artifact action to v4 ([#164](https://github.com/gbbirkisson/spis/issues/164)) ([9424bd0](https://github.com/gbbirkisson/spis/commit/9424bd028e880bdec41af6828ab97323c0e1c1a2))
* **deps:** update dependency rust to v1.74.1 ([#166](https://github.com/gbbirkisson/spis/issues/166)) ([4ac7933](https://github.com/gbbirkisson/spis/commit/4ac7933ba595b2de69209c94472eead1018c721d))
* **deps:** update dependency rust to v1.75.0 ([#167](https://github.com/gbbirkisson/spis/issues/167)) ([4cbab5d](https://github.com/gbbirkisson/spis/commit/4cbab5dc5a749fc9da288d9bf635c225ab027445))
* **deps:** update rust crate async-cron-scheduler to v2 ([#168](https://github.com/gbbirkisson/spis/issues/168)) ([f7cb8c8](https://github.com/gbbirkisson/spis/commit/f7cb8c825438f63e061d66ade227d2ace152fee2))
* **deps:** update rust crate image to 0.24.8 ([#171](https://github.com/gbbirkisson/spis/issues/171)) ([9c8f1b2](https://github.com/gbbirkisson/spis/commit/9c8f1b23074dad80f23fae611df806390eef665f))
* Update renovate configuration ([6975373](https://github.com/gbbirkisson/spis/commit/6975373ac5ba408408890b623b92e683d0a4996a))
* Use font awesome instead of svg ([#161](https://github.com/gbbirkisson/spis/issues/161)) ([644e590](https://github.com/gbbirkisson/spis/commit/644e590bb4b7e014714ee4a0971a2568472aa3d0))


### Miscellaneous Chores

* release 0.6.0 ([6f5f42f](https://github.com/gbbirkisson/spis/commit/6f5f42fbe3226911d87e7903b8745ce80cb11ddb))

## [0.5.7](https://github.com/gbbirkisson/spis/compare/spis-v0.5.6...spis-v0.5.7) (2023-12-06)


### Bug Fixes

* Add pre-commit hook for conventional commits ([fdc95e8](https://github.com/gbbirkisson/spis/commit/fdc95e86853bf4e67156a6230d52fff2ddb49182))
* Docker tag ([ddd832b](https://github.com/gbbirkisson/spis/commit/ddd832bfd456309781d9761cfefaf710f4311603))

## [0.5.6](https://github.com/gbbirkisson/spis/compare/spis-v0.5.5...spis-v0.5.6) (2023-12-05)


### Bug Fixes

* Add extra label to release-please PRs ([85952de](https://github.com/gbbirkisson/spis/commit/85952dedcde64aad944f8e0c9b2deffb3641fea4))
* **deps:** update dependency rust to v1.74.0 ([#150](https://github.com/gbbirkisson/spis/issues/150)) ([3ee6487](https://github.com/gbbirkisson/spis/commit/3ee6487b51f1474eec44f6fe8472ee3f21dccd7d))
* **deps:** update nginx docker tag to v1.25.3 ([#151](https://github.com/gbbirkisson/spis/issues/151)) ([bb830a6](https://github.com/gbbirkisson/spis/commit/bb830a6743b472ccea707beac8a640ab74cf3050))
* **deps:** update rust crate image to 0.24.7 ([#147](https://github.com/gbbirkisson/spis/issues/147)) ([3813a7c](https://github.com/gbbirkisson/spis/commit/3813a7c1807d7780cd2720b810fa54cd8ef00a6a))
* **deps:** update rust crate log to 0.4.20 ([#148](https://github.com/gbbirkisson/spis/issues/148)) ([84bbf10](https://github.com/gbbirkisson/spis/commit/84bbf1042cd9fcded981e16ca3052dcfbc742eb6))
* **deps:** update rust crate notify to 6.1.1 ([#154](https://github.com/gbbirkisson/spis/issues/154)) ([7e8b8ee](https://github.com/gbbirkisson/spis/commit/7e8b8ee925d1bc76d37739afe2b31e5d2c0397ac))
* **deps:** update rust crate sqlx to 0.7 ([#155](https://github.com/gbbirkisson/spis/issues/155)) ([01b79a7](https://github.com/gbbirkisson/spis/commit/01b79a7d1c2ec66c7e56f596c8fed9aefb230c09))
* **deps:** update rust crate which to 4.4 ([#156](https://github.com/gbbirkisson/spis/issues/156)) ([cefc6a5](https://github.com/gbbirkisson/spis/commit/cefc6a53103b27240b76e14754c04aee081fb082))
* **deps:** update rust crate which to v5 ([#157](https://github.com/gbbirkisson/spis/issues/157)) ([69521da](https://github.com/gbbirkisson/spis/commit/69521daf96a8ea2af75a6e7f9deed8c963ca836d))
* Fix release-please tags ([9a8a7f3](https://github.com/gbbirkisson/spis/commit/9a8a7f346824d61faf3baef1600dfe0dfd0cdf43))
* Run CI on toolchain change ([dfa3db2](https://github.com/gbbirkisson/spis/commit/dfa3db2de3b80af5114c39bb6d74b66163e05cd1))
* Update cargo configuration ([4039028](https://github.com/gbbirkisson/spis/commit/4039028ee68f17fc4a3fd25cf078da262f756346))

## [0.5.5](https://github.com/gbbirkisson/spis/compare/spis-v0.5.4...spis-v0.5.5) (2023-12-05)


### Bug Fixes

* Remove dependabot ([29fbc42](https://github.com/gbbirkisson/spis/commit/29fbc42722cc3570ad7051dcb730113eed40ae2c))
* Remove validate step ([047032a](https://github.com/gbbirkisson/spis/commit/047032a451c0ca46a77398aeca4878bc34477873))

## [0.5.4](https://github.com/gbbirkisson/spis/compare/spis-v0.5.3...spis-v0.5.4) (2023-12-05)


### Bug Fixes

* Add release-please ([94ccc2e](https://github.com/gbbirkisson/spis/commit/94ccc2ec0eec6bc8fec59ec6fd87a40c85859174))
