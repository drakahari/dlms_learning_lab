"""Synthetic guards only. Never execute a probe, grep, Bash or learner input."""
import contextlib
import copy
import io
import json
import stat
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import shell_ere_reference as c

PROJECT = Path(__file__).resolve().parents[2]
# Named CURRENT QA extension, not inputs observed by the sealed r1 ELF.
# r1 COMPILED396/SOURCES409/collector/probe stay immutable. Fresh r1 capture
# must still refuse these paths until a separately reviewed new collector.
CURRENT_NATIVE_QA_INPUTS = (('src/ui/smoke/ere_evidence.rs', 81617, '1a075a82595fa093ab5889ac65cd136f7ced24904d08ceae56163686c07e6d70'), ('src/ui/tests/ere_evidence_native.rs', 36060, 'd86a87aa0c705f6e9036416285cb1331adcb546825fb0bae7453684bea76a3d0'))

# Named CURRENT product-workspace extension; never sealed r1 compiled evidence.
CURRENT_STORAGE_WORKSPACE_INPUTS = (('src/partition_fs.rs', 38758, 'fe32a0d4fd68cf128cf047f79dd60ace55a64c582f1ae15e2a9e9243e6cf3dbe'), ('src/partition_fs_workspace.rs', 49240, 'a9fa08777ef4e5e70f4f08145e38f999e4360739151e6fe714cdd727b1050b0d'), ('src/partition_fs_workspace/binding.rs', 16673, 'fcf2da4ef2fc44a32f840dba537dd77eb35cdfcd3f0e5cc4abc73d3d6165be6e'), ('src/ui/partition_fs_workspace.rs', 52387, '4d0bf280b55da31d7032f0bfd8da9c4515e3dea52ac5963136b5a4e04cbf7fc6'), ('src/ui/tests/partition_fs_workspace.rs', 62096, 'aac1752f0289b6f99281423216383d4366286e8ae531d26fe7a265b38dd98441'))

# Fixed current authored Pack only; not observed by the sealed r1 probe.
CURRENT_AUTHORED_STORAGE_INPUTS = (('scenarios/disk-preparation-v1.json', 118384, '6971f6001d8cd9f3e58d28ef708ffecb6a5f4fdd1ad0f12b3d9606f07bf7dcb1'),)

# Two fixed conditional native QA modules; no sealed r1 observation claim.
CURRENT_STORAGE_NATIVE_INPUTS = (('src/ui/smoke/disk_preparation.rs', 159760, '46dce80e0335b396855e3fba1fc73acf064b359b56e3b3a6e44303665ca8f06d'), ('src/ui/tests/disk_preparation_native.rs', 19996, 'f5aa52b06996440df8f7b66417ca47b84944d8d68a7e6d1c44b53f587a2728ad'))

CURRENT_LITERAL_PRODUCT_INPUTS = (('src/shell/literal_trim.rs', 9256, '9917307a858e06a7592811b49b466f9a5211a303cf915422a16520b83cea3f69'),)

# Exact unregistered authored Pack for CURRENT QA only; never a frozen r1 input.
CURRENT_LITERAL_AUTHORED_INPUTS = (('scenarios/shell-literal-normalization-draft.json', 42234, '8dbbed5fa3de2da4e01de5694b4ec0e1b4269f8bdaf42b02ebc32e5ef40ca823'), ('scenarios/shell-snapshot-rollout-draft.json', 46123, '6d8cb5a647b702594bb45a83531ec95d90c351566ba68572a11908c0fde1aa0d'))

# CURRENT HSRP workspace only; never sealed reference evidence.
CURRENT_HSRP_WORKSPACE_INPUTS = (('src/hsrp_workspace.rs', 24959, '5f4b2c3f1aee47270b0607aabbba724dca74189b5852fbdf0d738327679b2413'), ('src/hsrp_workspace/binding.rs', 6020, '19d9e120d7ade1e920181b137bea53f57ad87641c78ddee9fc5b0b426db8fbb6'), ('src/hsrp_workspace/goal.rs', 16815, 'f32f2ca00ff1c4590f4c5712885bd0f597ba6fe95b63e6171d82c3af6d3cbe8b'), ('src/ui/hsrp_workspace.rs', 45102, 'a9ea13a503535b9708b79cd264d69bbdc8a904b9d55f0ff044118c5b8c5ad02c'), ('src/ui/tests/hsrp_workspace.rs', 53898, 'f2b4ea966c83bb261f2c187bd73b0e9f581dff7ddc7e12473e48da2e0291b8cb'))

# Current authored HSRP only; no sealed-source or integration credit.
CURRENT_AUTHORED_HSRP_INPUTS = (('scenarios/hsrp-resilient-gateway-v1.json', 103880, 'f8db2ea4a4391fca0dcd8a771404086f484526431ce092832b1e48b67261617e'),)

# Fixed CURRENT native QA only; sealed reference inputs stay immutable.
CURRENT_LITERAL_NATIVE_INPUTS = (('src/ui/smoke/literal_lessons.rs', 43269, '4dc77c0f64db5caa4e112f7854e6fb19bce13231e7cc95fdf5424e1bb53e19e4'), ('src/ui/tests/literal_lessons_native.rs', 31694, '955b1033ea58893464109091bcdf736c44fa04d26d5b7189bbcf5b82a4669756'))
CURRENT_SNAPSHOT_NATIVE_INPUTS = (('src/ui/smoke/snapshot_lessons.rs', 43204, '580f43ab9b14b43c3bce6d28597b82db3572fb4f6cf2b886f7c35d333ba7e4d7'), ('src/ui/tests/snapshot_lessons_native.rs', 13514, 'ac4e370f045de5ca557a793759e2782e8f0b0788b93ee7a39a0e2759de4fb2a2'))

# Current-only exact Syslog extension; frozen collectors and evidence stay immutable.
CURRENT_SYSLOG_WORKSPACE_INPUTS = (('src/syslog.rs', 39073, '19dcc8eb8f4f70b8aa701a31d9e974d88af2a20b00ecb2d35ed5db51208f9620'), ('src/syslog_workspace.rs', 47197, '016863b8f36f68842bbaad56e3f6dcc192dcb28584ef0311e545ffcb1d215112'), ('src/syslog_workspace/binding.rs', 5504, '374249975862d3232f27c876d21368d46bda727084ed25abb1cfd60bdf4a6f9a'), ('src/ui/syslog_workspace.rs', 49717, '70edd540ec59589e660b5bdd6132b58ee0a9c0c1d432f38fcc76e283f036137c'), ('src/ui/tests/syslog_workspace.rs', 67465, '767791fd017bb9cccf9a7000cfe5f33dd094d49ce7a7e5c636608330935e009f'))

# Current-only exact Syslog extension; frozen collectors and evidence stay immutable.
CURRENT_AUTHORED_SYSLOG_INPUTS = (('scenarios/syslog-policy-v1.json', 121153, '832b56f44e8f94a199516695589a6668ea3cb03c2c36731cc8e49ac46749ac59'),)

# CURRENT review-only sources. Fixed complete list, not a runtime directory allowance.
CURRENT_TEACHING_LINKS_INPUTS = (('scenarios/ccna-magic-number-v1.json', 12454, 'd5cf817975a765ee16b7a5556e533dc56fd428670f2c9ac44fa36c0ba464150f'), ('scenarios/ccna-magic-units-v1.json', 447, '0457682938e7cd9f335035d1f243dca50e61401a45cf4edaec38284cb043a1f5'), ('scenarios/continuation-units-v1.json', 505, '95eb52c3eafc69532ca7c60e31ffe7a00a2eebd7a0cfa67bf1f07df2bd04190e'), ('scenarios/hsrp-units-v1.json', 587, '7ff1580b20d7994d36c93e460c24c027972649ffece6f1f7aafc973c57c4bbaf'), ('scenarios/links-lessons-v1.json', 36563, '58e62fac807915e7d49bee730ef1d735e70a54ff1dc16472c83bc21a59c97dd0'), ('scenarios/presentation/bash.json', 279854, '8621163c759b262fb4bef0b02d730c226aa7b925aaada3f6e9b4eeabc807d3a7'), ('scenarios/presentation/ccna.json', 476916, '12f76b18f9b766e68b28fcca26f8c0c31701d78a484b965689d7bd78521198a7'), ('scenarios/presentation/lpic1.json', 774233, '5ac6a5ffad9d2697ff71b479a34a6b6b05385dea79510cb6b2a2e59b43239111'), ('scenarios/presentation/lpic2.json', 885724, 'cd25f20bf4e0a55885ac8f11b0b50a416070c12c5f6bd926da738784bb380ecc'), ('scenarios/syslog-units-v1.json', 530, 'e2c6b814ac00f340bc4d71122cf6bb19a1704955fff06fda6f028e2f1d02d213'), ('src/links_workspace.rs', 21814, 'ad94ddf5e2b0d4a7a9ebff81fe45a277e153361502b967b8770f1d30088a023d'), ('src/magic_subnet.rs', 11409, 'bbd4f19918f3200dc85ed822976a0f46380e525a7cd2ce3652d923ba33f1fa93'), ('src/ui/ccna_presentation.rs', 1473, '8ba158fc31292cfbd80405b8dd63345ace8574f756f87986ad2d058e61b06d31'), ('src/ui/ccna_preview.rs', 17228, 'e47fb864d02e6b25aa8bdf7158a38610411711647007774ce71978b782d51422'), ('src/ui/ipv4_teaching_revision.rs', 5184, 'fda67c8f111a449a444729ca0c8c9f6ec056f8233464d28a70726ed6c0760520'), ('src/ui/lesson_presentation.rs', 39757, '40a4737780741185b86b3e07aa4c7609d5481b0e0258f689ae5a717de2bddb16'), ('src/ui/links_page.rs', 16515, '41f936c960dd2935fb4f1320435cf2a9c2faf2abbde7bae48956c0a9c37041e3'), ('src/ui/magic_subnet.rs', 31226, 'c9b9c6e16a747e72969a45add901db6d01681bd874436864a2121e5b5c5a7ca4'), ('src/ui/smoke/gateway_lessons.rs', 14827, '64e094d0ad17f5212d1f7e6834454f5af52e5ed91071f8900851d0e1ddb3b154'), ('src/ui/smoke/links_lessons.rs', 25390, '95d75f22af8c2b69a8054e02ec2dd52f2e871e4b829e8f9a9b70ef31babfec16'), ('src/ui/smoke/syslog_lessons.rs', 22070, 'e4e0b09a2564bc724993755521e54fc18b4296959341fd74553dfab22ec6316f'), ('src/ui/teaching_preview.rs', 69674, '72628bac84f4e450b4790ae1ea6f5859615440c36189a1dd7ca17b8f13d7ca1a'), ('src/ui/tests/ccna_pilot.rs', 19516, 'abf493db8b182cb5e1ecda52f386e59a0b588a0919f05fdf8254f5e99168ad84'), ('src/ui/tests/gateway_batch.rs', 11584, '60fd994b5189f26d844453f9d3e68d091aabc39158d9a1a54e778d336b8ed933'), ('src/ui/tests/links_lessons.rs', 11121, '64bfc71a73a451f9587ec7788211efe7bb705b9395cf90ad5071d3e0734f72ff'), ('src/ui/tests/rollout_bash.rs', 11073, 'bf2f854b3b2f554e5d8c512c00f3f8a329109a0078c18f4218648e3ca3845aae'), ('src/ui/tests/rollout_catalog.rs', 16088, 'eaa55d88fa33445ef813e7eceb27ef623d22e61fc120425566de5409f3d8f1a7'), ('src/ui/tests/rollout_lpic1.rs', 6046, 'e2b7aed41a3bb89aa95fbd008c99b0b0cd3bc361ecb160c263bc7fb104bb3dc4'), ('src/ui/tests/rollout_lpic2.rs', 2782, 'bd918a0c23a1a0b2546e62420391441b250ab499dc03589cf303415ed3cf74ac'), ('src/ui/tests/syslog_lessons.rs', 3232, '58221a671e8ca9493ec5ddb64c9cca9084c42d263947461841a6baee3d794670'))

# Exact CURRENT-only pure foundations; no frozen capture input is changed.
# Current capacity lessons only; never additions to a frozen collector/observation.
# Repository lesson CURRENT QA only; no new frozen producer inputs or observations.
# Pure host foundation only; no added frozen producer observation.
CURRENT_HOST_LESSONS_INPUTS = (('scenarios/linux-host-lessons-v1.json', 102566, '8a94d70594e0a09c21dd936e1f0a8127b70e71987f80af9850c6ee24c089cb7c'), ('scenarios/linux-host-units-v1.json', 518, '186461e21abe6ec185c62052c593eb4ed239bdbb48ab8e7b3b8b3c3368b9649e'), ('src/host_workspace.rs', 26530, 'f6ca6a6d42fd375a4793892948cd2116aa0bbdb8a4920051bb85dfce6a0992dd'), ('src/ui/host_page.rs', 24919, '96c9da10fea8fe3607ea0a7ef169f026948ae320b8c7003ee68bfb72dbc5041c'), ('src/ui/smoke/host_lessons.rs', 25446, '42878b5d8394a160c291f800c00933614de3955ddc24504a4eb01748fce47a41'), ('src/ui/tests/host_lessons.rs', 21160, '6c63a07ece02bc5b60bac325355d0252bd3024e34f4a2673773524c7a03f589e'))

# Current-only direct-query expansion. Frozen reference rosters/seals stay closed.
CURRENT_DIRECT_DNS_INPUTS = (('src/host_direct_dns.rs', 7627, '0e6c8b09e3444ae903e316abf457a88f2d46f44ea5058e44b2de41c3e43276e6'), ('src/ui/tests/direct_dns.rs', 9528, 'fe1153d8aede7fc19f71a2a1f43e2bb7d101a92117a18721316fd409301ea72a'), ('src/ui/smoke/host_lessons/direct_dns.rs', 3323, '5ccf3275211fa0794ab4961df50d596a36bbd729bbface743e92423764d14c11'), ('scenarios/linux-host-lessons-v2.json', 102886, 'b1161e4227618d2c6ce56a78d9f220400673a4f1b08a4b6ebaf168eca9006cd3'), ('scenarios/presentation/lpic1_direct_dns.json', 6072, '0387ba076de361f1472f19555bfde597b3f1d8b6ef5dba0a7943abc229e41271'))

# Exact CURRENT-only authoritative projection; historical producers remain closed.
CURRENT_CAMPUS_ARCHITECTURE_INPUTS = (('src/network_architecture.rs', 19558, '0f4f354c684e838c47c5a12b5baf38cdb8d2038be22e81348dc836c0abbb6658'), ('src/network_architecture_source.rs', 3325, '47a29317c35d781d6e2250ac0d81e6125cf74ef235e6cdf7cf14a1c77c84c247'), ('src/network_architecture_workspace.rs', 16756, '81d6d63d4bfd8968456b8c1209b1a370e27c489a9cc5b43f62ea2205bb742048'), ('src/ui/network_architecture_page.rs', 28951, 'ef8fe22a3f9b368679b70d5eb92255b0c1371b123cdc42ac477f4b89ff2c42b0'), ('src/ui/smoke/network_architectures.rs', 18166, '6577c3dbc87d7830d7dfdf0a4afa90a626acbde17ea2297d0ab0d782a8e56a47'), ('src/ui/tests/network_architectures.rs', 18320, '2819c2e42ae0f32c484a418b348e3ae4549ec5fe4e3aae4d084bbe96ca538132'), ('scenarios/authoring/campus-architecture-v1.json', 62803, 'c58b43fbd03f6c9e04ee872316f979db351482b9f251fe17280162ae1b856e93'), ('scenarios/campus-architecture-lessons-v1.json', 62803, 'c58b43fbd03f6c9e04ee872316f979db351482b9f251fe17280162ae1b856e93'), ('scenarios/campus-architecture-units-v1.json', 512, 'dba515b9ec119d54b1822f9e1416426d7073dacf7b2fe2fa46aa65ea517ef441'), ('scenarios/boot-entry-lessons-v41.json', 1885800, 'f3706e6bf02502120deeb6a24dca97648a51c4c8b9c762f3f260a697838a12e3'))

CURRENT_LEAF_SPINE_INPUTS = (('scenarios/authoring/leaf-spine-v1.json', 33671, '77ef645f4d5467fcfee9d1ca189cd488b3d5a2810351d634fc435ff635e66453'), ('scenarios/leaf-spine-lessons-v1.json', 33671, '77ef645f4d5467fcfee9d1ca189cd488b3d5a2810351d634fc435ff635e66453'), ('scenarios/leaf-spine-units-v1.json', 430, 'be6a0a22b4559a64b1329ce5ed94d98180638bc351fd2e4a2281bf9dd9d7e0f9'), ('scenarios/boot-entry-lessons-v42.json', 1919424, '341c76706506524f930b20ee23e4be7313f05252d8c2999da867b952e3c5dc76'))

# Current-only bounded connection/NAT lesson extension; historical reference remains closed.
# Current-only full-width IPv6 numeric foundation; no new authored coverage.
# Current-only virtual services/filter composition; no authored coverage yet.
# Current-only pure IPv6 administration foundations; no new authored coverage.
# Current-only IPv6 snapshot foundation; no new authored curriculum yet.
# Current-only FTP identity capability; no new authored coverage yet.
# Current opt-in exposure capability only; original frozen producers remain closed.
# Current finite VPN module; this never authorizes historical capture.
# Current opt-in RPM configuration decisions; old collectors remain closed.
# New pure offline filter capability; existing collector/control inputs remain immutable.
# Current-only supplied DKMS model/content/UI; frozen collectors stay unchanged.
# Current-only early-userspace primitives; no frozen reference observation changes.
# Current-only PXE model; frozen source observations are unchanged.
# Current iSCSI sources only. Archived observations keep their existing pins.
# Current ATA SMART sources only. Archived observations keep their existing pins.
# Current Apache HTTPS sources only. Archived observations keep their existing pins.
# Current desktop profile sources only. Archived observations keep their existing pins.
# Current security investigation sources only. Archived observations keep their existing pins.
# Current credential policy and VPN design sources only. Archived observations keep their existing pins.
# Current staged-network sources only. Archived observations keep their existing pins.
# Current network evidence review sources only. Archived observations keep their existing pins.
# Current site connectivity sources only. Archived observations keep their existing pins.
# Current host and DNS security sources only. Archived observations keep their existing pins.
# Current VLAN gateway sources only. Archived observations keep their existing pins.
# Current NFS export operation sources only. Archived observations keep their existing pins.
# Current installed route reading sources only. Archived observations keep their existing pins.
# Current broadcast designated role sources only. Archived observations keep their existing pins.
CURRENT_OSPF_BROADCAST_FOUNDATION_INPUTS = (('src/automation_workshop/ospf_broadcast.rs', 110, '1f66d92e02eed72163172dc8064f3c3947cb56324b89be23742865faba18bbed'), ('src/automation_workshop/ospf_broadcast/election.rs', 4885, '3f898409268e2e374d398f93b4cc9c19ed1aa4e30c159ec12c587dc39f167cde'), ('src/automation_workshop/ospf_broadcast/workspace.rs', 6952, '6fb3bc3b69bd3fa6c0dc5db657f8bec4b7dcc3380a997c5c235cf388829db7eb'), ('src/ui/automation_workshop_page/ospf_broadcast.rs', 4467, 'b327b6f2c9bf8a1a11416cb50722f98afe3eba5397d21f1973a3a175cf3ca0f0'), ('src/pack/retained_542_catalog.rs', 8722, '13b05d9cb75bbd97ca0351ca24f0b776935452edcdf093fb3f311b54635db2ce'), ('scenarios/ccna-ospf-broadcast-lessons-v1.json', 7762, '3b723fa9fd1659efce80d704e57e9f389fab057b4ac346d208b78613c7da333e'), ('scenarios/ccna-ospf-broadcast-units-v1.json', 436, '9a229211cb62c7e10dab8e677833a21677d175aed274993ea587548a21f9dca3'), ('scenarios/presentation/ccna_ospf_broadcast.json', 3872, '5e71ad54618274a8d6eb97cba557e206340b3381711174acbee76ef58391a3c8'), ('scenarios/curriculum-continuation-v19.json', 892788, '5c33c1fffae0873522ed4f9f42c970e9595d4b1cc2b54b1d52694afcd8d33418'))
CURRENT_ROUTE_READING_FOUNDATION_INPUTS = (('src/automation_workshop/route_reading.rs', 105, 'fc22c8fdae7740fb789a3adbc92165b65ce8484c1562dde2bb25744d1b9920c4'), ('src/automation_workshop/route_reading/table.rs', 7032, '27406f5140361acab4754a14c763cbdf6db5ec17ba499dc41cdac5246ed01375'), ('src/automation_workshop/route_reading/workspace.rs', 15319, '9b7b120dfac6e0601b87c7003b3c586f5f4197d23cf5f099fcd967fa0bb1a28d'), ('src/ui/automation_workshop_page/route_reading.rs', 8480, '0096d77e66a50138af1e54f807ea775a3197c01e0a8027adaa6100ff62a20fc9'), ('src/pack/retained_541_catalog.rs', 8713, 'b712b88d1651ea44b07e8b8a035e59cc36fe34492f4d6e0cee7cf7ef1c7f400b'), ('scenarios/ccna-route-reading-lessons-v1.json', 38961, '96b890f0eba90b749feb4e7d555fa0d2e6d32444d4f6ad3d1c388ef2b3237411'), ('scenarios/ccna-route-reading-units-v1.json', 428, '0507ba8a46dd804b98979117fde4e717ad5ae499c522ddc0579b03d457966254'), ('scenarios/presentation/ccna_route_reading.json', 4313, '33d2e92e2b252da8756ebbf3feda7642e98dd43c392ad0e04f4e89a7a011fec0'), ('scenarios/curriculum-continuation-v18.json', 885073, '4b1d67e2d8debb74edcd7f8ab1fcf0d6fdd00c9bfb156adbe730e840f485ec5f'))
CURRENT_NFS_EXPORT_FOUNDATION_INPUTS = (('src/automation_workshop/nfs_tools.rs', 79, '7cf5cac87eebb7dbfb46498c748140ef35402be0b292d0f7ce9d6296e96fa196'), ('src/automation_workshop/nfs_tools/operator.rs', 5040, 'dd8ed5dc41388b499071223e6818824530887040d281080d3af0e2eab4555092'), ('src/automation_workshop/nfs_tools/workspace.rs', 13224, '61c6140d2316e2e4f270c458f6f9e65beca6f005b3f282826e55a53a19f704b8'), ('src/ui/automation_workshop_page/nfs_operator_page.rs', 6654, '733b8e7b9ab3062fe7f50c49d660a8cca04ba9cc6c76cb980798d5b6574cde51'), ('src/pack/retained_540_catalog.rs', 8741, '3911b94058d9dc11890ea0645045234670c7c8b3962036c3e91b20dcd3d69e74'), ('scenarios/nfs-export-operations-v1.json', 76450, '1451994fc078b5c031e59c7b4de1b876457d2caed8024e56c6c0699f881e9a49'), ('scenarios/nfs-export-operations-units-v1.json', 447, '0588319213b641d6bbb86db3e69066062343d4eadb95127859ccf1606d831daa'), ('scenarios/presentation/lpic2_nfs_export_operations.json', 4092, '85ddc50d57b3eccffa82e14b181dfb26d154140d682cebcef59e7d7fc0fd360d'), ('scenarios/curriculum-continuation-v17.json', 846159, '1c96156c7f81f2459d0ef58b1f093ce75fe0bc3309f978b317246fa4b834fe22'))
CURRENT_VLAN_GATEWAY_FOUNDATION_INPUTS = (('src/automation_workshop/access_defaults.rs', 6652, '30c5598e8796508218832b20b756e4dfd4a48cf2f4ea3af363287edfad64d789'), ('src/automation_workshop/svi_projection.rs', 17658, '470066834d300654106b65ce632d23906a7680bc477f6bb845c4fb28801a50c6'), ('src/automation_workshop/svi_draft.rs', 3593, 'ccfece6bda06782fba226358b95fb57fba654d7b078ff3b581da92cc0d5d560a'), ('src/ui/automation_workshop_page/vlan_gateway.rs', 13084, 'a12a60d330e5aa62ea1a70e6b5d7965b3d584e13d4618be29fcba4b87b7e12cf'), ('src/pack/retained_538_catalog.rs', 8221, '417dca6f884da558e2c28c3c470a648b91131b706a33e5cb0d26a7f2fceca8ad'), ('scenarios/ccna-vlan-gateway-lessons-v1.json', 41764, '3bd012ca39ddeb4328a9af16b6f63115978553916b66c5d76a3f6d382d1612f6'), ('scenarios/ccna-vlan-gateway-units-v1.json', 518, 'e2445dc33ee8471022cecd2d2c6acf7084ace2a921a7edc9e2f322a04a1f5852'), ('scenarios/presentation/ccna_vlan_gateway.json', 7606, 'f0b70dda9eff550ccde2691fa1a302bd76a7370282527fadac36734c7da52a91'), ('scenarios/curriculum-continuation-v16.json', 769756, 'd521e91748e6b5bd2628965623a3e4f0185de744a4862be7355bf6dd6cf4e502'))
CURRENT_HOST_DNS_SECURITY_FOUNDATION_INPUTS = (('src/automation_workshop/authenticated_transfer.rs', 9557, 'a0326295405fc91578be4f5693f25d87aa5dbf439e4ca210fce05f9d2bf157f9'), ('src/automation_workshop/retirement_console.rs', 4529, '46edf601bf70f7a483aa8dd8242e99c0f53e6f8ad0205d27a92b76d52d2ac826'), ('src/automation_workshop/socket_retirement.rs', 8139, 'ee1d3362202208e6d167dd844a800c8800dc0670ed2311adf5a6f9912ee78c02'), ('src/automation_workshop/socket_terminal.rs', 4203, 'a3fa8ebd694fe42849ccb129b97704fabfe6a977dd795f17ddd7692fc3911aef'), ('src/automation_workshop/transfer_editor.rs', 8778, '92216bcbb5ffc31ff09d6fde5fd1eab6b53679e6ca541ded2ebb7e7f35594344'), ('src/dns_access/config/transfer_fragment.rs', 1688, '88d799df75d81d77f84a69faa0fe3ea8a31a12a355076e0e9612ecb918a6a7d9'), ('src/ui/automation_workshop_page/host_security.rs', 10977, 'd2900291c4c831b606f197aa6d62596dbd2c98c0cf34910196be8bd63222ad26'), ('src/pack/retained_536_catalog.rs', 8209, 'f7911f04d77fbe17d95be22033e6f7997987461b9dff4dc93b0867d01b5582b4'), ('scenarios/host-and-dns-security-lessons-v1.json', 56745, 'd0f4c3e5378b935e51797beed069c8307d25e46a043d2719e2e902ab6d503a5f'), ('scenarios/host-and-dns-security-units-v1.json', 858, '84e4268a5b454df9d8bad4b848562960f9f89f75c8ec6e47004b0cce2911e878'), ('scenarios/presentation/lpic_host_dns_security.json', 7659, 'ed678d8a89b00d60996488873922b6a93c92056e8491f62e54562e075841b602'), ('scenarios/curriculum-continuation-v15.json', 728039, '1eb15bd21e2744ab29cb02edaf241fcd3d5d39aa6fa350508ccb7a0a65934d38'))
CURRENT_SITE_CONNECTIVITY_FOUNDATION_INPUTS = (('src/automation_workshop/site_connectivity.rs', 5785, '534cb978fdd88df670dae394bd86700565a1887cbab0649d97e6f5aa35e348e8'), ('src/automation_workshop/site_connectivity_task.rs', 5224, '04552a5f2aa93c28ffdbdb23577892a519573b404bc88b8fa83d6b35735e67a8'), ('src/ui/automation_workshop_page/site_paths.rs', 6299, '1dff8792ee415485f09e52eff20bb40505a076f2f50f46996f9194d03878cbad'), ('src/pack/retained_534_catalog.rs', 8215, 'e92aff717873e2e816134ac4f00d4238918327ac2172430668a7d4744bedbd8c'), ('scenarios/site-connectivity-lessons-v1.json', 45954, '7fdceb8a2306e01001271b2484e6da5980054e4fed606caa2cd47dda19eada07'), ('scenarios/site-connectivity-units-v1.json', 471, 'c4e8f42e3d7ff3da21a744433fd606c34524f05d8d0657474ae71f5f738dc241'), ('scenarios/presentation/ccna_site_connectivity.json', 7080, 'f95bde543348fa395edc7f1f48c355b85babc20a8e3fef9abae79dd0c0b78226'), ('scenarios/curriculum-continuation-v14.json', 671341, '60e3069ca0110a66ac314b0396e57877f0b60812b1e1bc497c6ffcc193b404c1'))
CURRENT_NETWORK_EVIDENCE_REVIEW_FOUNDATION_INPUTS = (('src/automation_workshop/claim_review.rs', 9326, '7f30a99e4358a8a84cf21efce391cf370cee531480ce08ae32919d7eae41aa87'), ('src/automation_workshop/desired_config.rs', 9469, '40ee6486822dd4bd51e915fd366a34d7f9d1376b8e6123238b83d241249b8213'), ('src/automation_workshop/plan_editor.rs', 2802, '484da996177e3e926360dfa6312080c7cee4067e51159a6ca1616f1ed0d965a5'), ('src/ui/automation_workshop_page/evidence_and_plan.rs', 11104, 'd4f4beafa1a2d580600d5eff31737ebadba3b929abf6726aa1e15c663e62fb4a'), ('src/pack/retained_532_catalog.rs', 8206, '0284222f99f3d5c8fd70544a340683e4141e4751e26a871a28453ffed72f9b62'), ('scenarios/network-evidence-review-lessons-v1.json', 67930, '87a6c1d4b57e1e3f1c1df445e7a938e834a32910593e39dc70bed80c5a93dee9'), ('scenarios/network-evidence-review-units-v1.json', 534, '99d83cb54b71b493e1b07ce1a5eae662846bc107a73a9406e987b33127cf8b42'), ('scenarios/presentation/ccna_network_evidence_review.json', 8222, '395889d79cb2ab178c217bf00777c1397daf992d81a2de9e71565bdbf5ba21aa'), ('scenarios/curriculum-continuation-v13.json', 625434, '7f7ff5b5284551f8782c9b19f5edc642fa10ffd8b7b8c1188018a8d6ca711353'))
CURRENT_STAGED_NETWORK_FOUNDATION_INPUTS = (('src/automation_workshop.rs', 647, '4d1d1d6312d483af1bcba9ef3f2c6bd211028137ae630bf8a43530341c01fb0f'), ('src/automation_workshop/change_rollout.rs', 8000, '654c73dc17ad438b5a6c25b395cfda1f8546c49eb3da7aa6e8b1b93991d1fae7'), ('src/automation_workshop/intent_path.rs', 5194, '706e66efe60edbd542b8d992c9fc4f1a273621e5582acf6c4064c9e146dcf97e'), ('src/automation_workshop/rollout_task.rs', 7477, '38eb9c13e54bb3ea102a77d37c5b2f953e77a4e0353653b693d7b98f8a639178'), ('src/automation_workshop/session.rs', 4274, '2d0ac14bc729c91608031faf01317df64170a6f1d722231a6a1ce36b1f1b7933'), ('src/automation_workshop/source.rs', 3458, '6d702f839018da37c6034ac671244c4644fdfa40cc333dc9d24c3525f346dd98'), ('src/automation_workshop/workspace.rs', 31594, 'ec629d467824c25d80a78546158956e4e2878b0a460089f6a9d7b0086f391237'), ('src/ui/automation_workshop_page.rs', 25256, '7c90fa1904b52b2b129973afcfa707b78a07911b8422bfbd0b332d36b7d377c2'), ('src/ui/smoke/staged_network.rs', 58293, '0e5b0a214454accac6ebce7af06cc82cdc46d7b3ae839dd9d75b6c8cde5b7527'), ('src/pack/retained_530_catalog.rs', 8210, 'd7336107bf5f2f6ec7737cca59fdad78a0a520a9167e6b464d62c41fc33c14d5'), ('scenarios/staged-network-lessons-v1.json', 54339, '5cdfb9047f047962241706aab80a761862916e10f1a9053b210035238b249bb5'), ('scenarios/staged-network-units-v1.json', 528, '6a6ce338bc872af0ad04316e914fa02d8b6f54434469a291c6bda4391b45627c'), ('scenarios/presentation/ccna_staged_network.json', 7911, 'a65418d12a996be71447ba6c33fc6de17f34444ecef91e49c1f6792ef5a4a69d'), ('scenarios/curriculum-continuation-v12.json', 557551, 'a6de21185033c3c260a3d336d6f0d50b944011747c3f1baca5516196972cf623'))
CURRENT_CREDENTIALS_VPN_FOUNDATION_INPUTS = (('src/security_investigation/credential_task.rs', 5154, 'e62c863e6f8a6ad23c904fba5d771cc00e89f08f5c1fc35dbcd4810dba5dfd17'), ('src/security_investigation/credentials.rs', 4588, '80d374e5a659b89d4b494ef682058770a8667d430e49cae37c8a234ffe1d5e77'), ('src/security_investigation/vpn_design.rs', 7570, '00b312307592a3121901d9947174033a9053ca409f0a36a489b36144284f17a6'), ('src/security_investigation/vpn_design_task.rs', 7544, '020500f25816071daa49873348df98d68efbee9369d874123e13d56fc9af96a1'), ('src/ui/security_investigation_page/credentials_vpn_page.rs', 9348, '8241bde6fe40639b3c0c9562c867aa813087a51b62d4e89bf1ef471ab0ac5b47'), ('src/pack/retained_528_catalog.rs', 8214, '35e4c366cca4cf00c145dbb609160df8728317b9598128f53562d9cb11a17b89'), ('scenarios/credentials-vpn-lessons-v1.json', 38346, 'd8b11b41fcf3e9526dd394bd534aa1ab15ac6e3d91bdfa975ffa4f05bb0bc8d6'), ('scenarios/credentials-vpn-units-v1.json', 488, '888fb3dbdf86b669e4ca4de55fe33b84cf3e7d0255d95021e1f2875880fd064c'), ('scenarios/presentation/ccna_credentials_vpn.json', 8035, '04f8a4bc65d5af1730d4d02197c51359abdbfe2032b403324ee0cddf31b3bf49'), ('scenarios/curriculum-continuation-v11.json', 503259, '26a0961d3330858b0581a4b3be10b82cc040dda20a5e31accfa3fa89225d2d34'))
CURRENT_SECURITY_INVESTIGATION_FOUNDATION_INPUTS = (('src/security_investigation.rs', 303, 'bb0465435943a7d0e7a3ebb645ac4d93364540c2733f3fb9b71971539635daa9'), ('src/security_investigation/access_plan.rs', 3354, '2a6f93a000e9b117fdf7362eb00cb13f0ae43e0f841353b6b8ef75645c7c640f'), ('src/security_investigation/change_evidence.rs', 6593, 'c62708b3b9d9b3b229d37d953a7c5e7261611c6291cf40a78d0f9ebcb417ee40'), ('src/security_investigation/session.rs', 4052, '57b794570afde82a0edf026c394b97f34d6b558bd9b00b80bb180d6b849b97ac'), ('src/security_investigation/source.rs', 3380, '21909f25123b3c47457abef00d8ad5bef90b036ad35f4a651640cbb2204e8754'), ('src/security_investigation/workspace.rs', 19501, 'fa412b776bd0ee306e549d5589ed25e9c5360e6d9c4ef6860ab415ecc2face1e'), ('src/ui/security_investigation_page.rs', 20496, '544dd54a6adadad879dca62716ddcb6a0ff8aa59ef5fb222ad2b3fdbdbc5635e'), ('src/ui/smoke/security_investigation.rs', 25272, '1acd57ccd25a68f92f311dde8139ef483c0b7fdfcf73447e2908a54335520631'), ('src/ui/tests/security_investigation.rs', 11087, 'bec05059da6bdaed71a4b9c37e5d58374b8d2a16f9db6fd911f7cfd1c9ff93c4'), ('src/pack/retained_526_catalog.rs', 8202, '5ca4fa353d904c693711858b9afa3be212fd2fa6191d7471c5d3938de597874f'), ('src/pack/state_validation.rs', 13969, '31b30c7ffee42210dc7d358c6701c1cd982443ccad467751e343fbe150da50d4'), ('scenarios/security-investigation-lessons-v1.json', 51525, '40881f76aa8ea0d4d6bba993c8f30b19633c2b6aeaade368a30ed2a38cf170da'), ('scenarios/security-investigation-units-v1.json', 513, '7ec0e33453fbe9ae2a20c8b79c8aecb2d4bbac973d27acd43b7ce37fc7e39640'), ('scenarios/presentation/ccna_security_investigation.json', 7993, '699279b9c1a114c676aa62d161fa9ae1a6f29000326076a126fe585b6d2bf578'), ('scenarios/curriculum-continuation-v10.json', 464960, '3acc75f34f03c0b18699a1df2f921454f304ba6079a9f002dc120126e1bb2605'))
CURRENT_DESKTOP_PROFILE_FOUNDATION_INPUTS = (('src/desktop.rs', 223, 'c3888898cdb00b0b3cad3411c97d54c8ce5ed756a9cf3a3a1570efb0c55a2c71'), ('src/desktop/xorg.rs', 7696, '946dde3446ce1c582f9595e9fa155533f219eb42fb51bc4881bcd4e28fdaae49'), ('src/desktop/remote.rs', 3505, '15462e9f13bbbfb77716b4498efc442ad074903dedc733aa43c288051999eb01'), ('src/desktop/typing_assist.rs', 3018, 'e363e4899f9208deb69ea09665ee59a8440eb8a9459a17c96fb3c36fe460c9ad'), ('src/desktop/workspace.rs', 15388, '4303610d79fc690a61c717e3a5885c0bb4c4d937d1b697177405e94bff7142f4'), ('src/desktop/session.rs', 4018, 'ad3308b090de00eb2a264adac670392be7a25d84ced84c2d6a0564aa8bd28587'), ('src/desktop/source.rs', 3251, 'bec290d66c50308bc178b08ea7eb0d2ed23fa801dd9e8a21b6d373c0ca8f714f'), ('src/ui/desktop_page.rs', 19232, 'f2f2a7aadf37d02738257009f16e6667b1bf55a63015ce1819c37ad09e36e0b9'), ('src/ui/smoke/desktop_profile.rs', 17053, 'cf763a48505158b017b4c090405e8ad1c15e5f67c3d75ca496c7fcd7925b3574'), ('src/ui/tests/desktop_profile.rs', 5403, 'ec321e9f085a7287fb2e3c8508a59e74ba95a32194f2d04b3b9622830e50cb8a'), ('src/pack/retained_523_catalog.rs', 8217, '6a41ad23e4be1b7d8819c5d5c40b5dbd85ea440635ab702e092a51a6e38cac84'), ('scenarios/desktop-profile-lessons-v1.json', 23639, '6659c676f90d7d6af1327fd5e2f8780685792437985c9836404294352d86351b'), ('scenarios/desktop-profile-units-v1.json', 520, 'acf5a81f2b83a343a74f224d2d9d84a5c15ab52aa0f1b3da28a773faf14f33bb'), ('scenarios/presentation/lpic1_desktop_profile.json', 11283, '6b2a3d2b7c3e2e1b540bfadd51749eabc94c82325ec496e69367db9ba22d143f'), ('scenarios/curriculum-continuation-v9.json', 413482, '62b05edfb2ab05613d2f02efb4f899024b2137b647f62ffb0ebb02185eafe2c2'))
CURRENT_APACHE_HTTPS_FOUNDATION_INPUTS = (('src/apache_https.rs', 10293, '93e92b1b9c4165418b858c0ff76ee3a0fe793e58807d07220a6cf7b046d4a86e'), ('src/apache_https/config.rs', 6499, 'b37e615326a1ca399bff2bac2df8af41e981c98053878916d88f8bfd5be687d5'), ('src/apache_https/workspace.rs', 9269, 'a9721fff2a457d15dac716e068bed42dfd0309cedd4f485e940173412e08f872'), ('src/apache_https/session.rs', 3934, 'c1e911670580c87155f1c87a3dd9943731da244d1878201a707d5b7e95582a4b'), ('src/apache_https/source.rs', 3415, '4fed3d001941cc02b2d35ba71e7a84541914e0f56ca3e6f5447868e78cd4f09e'), ('src/ui/apache_https_page.rs', 15769, '8106b484dd21c7115b0bed9c6b6ea671a9258104fb4d2def32d6c8b20d350792'), ('src/ui/smoke/apache_https.rs', 14605, '0a8ccdd879fc949bb45a07a384c4b852744c7f29b2f56a00f3a6a31f8c0f2e57'), ('src/ui/tests/apache_https.rs', 4883, '8c61b294fabcf41d05bba1740873764ebc623986ecd66df29bc35fc69b42e8e2'), ('src/pack/retained_521_catalog.rs', 8210, 'd333c2ac2436dd0bd3a81c86066e24c7cea8593ee76934afea053643e8aae49e'), ('scenarios/apache-https-lessons-v1.json', 49814, 'c27b4509329a347ec97faa25b95ae31bdb15df4ef2a3c9ae73846a0795bbfdf8'), ('scenarios/apache-https-units-v1.json', 409, '9b23a2cc16a495beaa1e6d489d0541f1c08a7aa6039df5600a4bac0a8989dae1'), ('scenarios/presentation/lpic2_apache_https.json', 8562, '5eba9a3c64f8aabf89e6bdfd439711fd087aa74c9a557aee493c2f3aac1247e5'), ('scenarios/curriculum-continuation-v8.json', 389890, '8590b295f222b77e71b633395353661d1573686886b69bfccb3bb4c978cfb498'))
CURRENT_ATA_SMART_FOUNDATION_INPUTS = (('src/ata_smart.rs', 13312, '6429b591023490605f7350f9851875c6e6ebb47c5dfddd515372d590ccbf5370'), ('src/ata_smart/command.rs', 2470, 'd7e230c9b080f3691d4dfbd971fd313d452be56411793797d2a406fe9e696cce'), ('src/ata_smart/monitor.rs', 3245, '6f41331ea39a488b397de8229c0c561e6bca915301b6378cd9cc2d5b76518bc6'), ('src/ata_smart/workspace.rs', 15166, '5f2444ddd54c77c5e4cf91eea9c05bc18dee2bd57f6dbe7ec2fbc76a1a292f56'), ('src/ata_smart/session.rs', 4057, 'c86620b99674cbe0ad484ed1fcd9aaa4dc3625545a6c2c8c82089f08f39e0f99'), ('src/ata_smart/source.rs', 5244, '8d3380ec5df0890dd4c723e8f1cb9c7317acda01805dc43dcce3a220abcd2e67'), ('src/ui/ata_smart_page.rs', 19414, 'd97679db5352373aaf91b45a1c5988a1e94b5873a07a29a73434f7ae12acf319'), ('src/ui/smoke/ata_smart.rs', 16349, '2d7a01c54497de066beb4fb2169a16165d6faaab3fd59f4d4dbaa48816f64f80'), ('src/ui/tests/ata_smart.rs', 4879, '4f06061929fe4eba967dafdae9d31ee8d87435d90c79155b8aabc6740acb21c9'), ('src/pack/retained_519_catalog.rs', 8739, '92e5440562b1a23beac7aaaf30328a1a589f3b067cd655c030e16292141e5ee9'), ('scenarios/ata-smart-lessons-v1.json', 24566, 'b7ab970366365d6bc87958307eee1d50babb24c120cd6a166d401dd397379b2a'), ('scenarios/ata-smart-units-v1.json', 477, '2dcc7778eea1d60e2cebf1b30c88606e301db5998dd0b0c60a0cf3cb38681c31'), ('scenarios/presentation/lpic2_ata_smart.json', 8430, '857315d07ca17be9ae28206492add60c020d2b4bb9faf439fcf155e8238cfbee'), ('scenarios/curriculum-continuation-v7.json', 340123, '828c46fe6c3ddaae350f4d2d409c44f29bc83bb73e0324943513618d94377e2b'))
CURRENT_ISCSI_FOUNDATION_INPUTS = (('src/iscsi.rs', 15603, '3444123829e2e199e42adca4b38dc7db88c50de38863c386005b43d214c633f9'), ('src/iscsi/command.rs', 6846, 'ab696eed07142d323e20b2dde292659b001379f4e26b3d26796c4b2893b5bd40'), ('src/iscsi/workspace.rs', 9742, '113b5f709f6dcddea4fa037820113fe19c90586ad33ac8a40f5767d7e12f66b4'), ('src/iscsi/session.rs', 4194, '42703e60f4d136be9ddacb35ab495fd421e4dcddc912d3ea3b99df4890e54ba7'), ('src/iscsi/source.rs', 3285, '8f8b996cba0f7e3cc11fc7bdc12cd9578f16026593ab235f13d5f85dde32e07c'), ('src/ui/iscsi_page.rs', 12507, 'de39026309b92ea2aea827aa1cb547c4a94ca89c8e50a33018efff390ebb2bd6'), ('src/ui/smoke/iscsi.rs', 13722, '9dae6d91e322df0e6f24dcf79394058d20d268cb32c9c88f16bfe268daf555af'), ('src/ui/tests/iscsi.rs', 3559, 'cd5c2179302df0343b8a283dac28617e814bd2d0f9be25195abf14f40c13bacc'), ('src/pack/retained_517_catalog.rs', 8739, 'd2fec57516070162128f089cd5e04c8980ed4bde1046d6db2c453517fda67b05'), ('scenarios/iscsi-lessons-v1.json', 26352, '7c0ce1ae1a112a40be983b97b65f68576bd64328ddfc91f180d0cd4d480c8f93'), ('scenarios/iscsi-units-v1.json', 458, '3f43fa15c41d6b7a660db49b66b1f13c74f233cb357fbbb070bbc34b70a05bc1'), ('scenarios/presentation/lpic2_iscsi.json', 8896, '79de2b4025ce9dab71960aab0200f342b82a2c3b15c9828ec3a5326904aa066f'), ('scenarios/curriculum-continuation-v6.json', 315604, 'c48f6ec1e55693f3304490318646f996414d56f22515492c3a28bda3ad00c28e'))
CURRENT_PXE_FOUNDATION_INPUTS = (('src/pxe.rs', 12863, '4741818f4dd2a5740a812209a27cc0ba191650d02b567ecf341beaaaf7268dc1'), ('src/pxe/config.rs', 4612, '9e1596102318993df3a9ed907be1e4565f2feea0abad7b7ad4d9e008f6115afe'), ('src/pxe/workspace.rs', 22047, 'b600aa73f63e717401c2542d6234e339ba227c5b0993c3f32cbb0bc67ccbc24f'), ('src/pxe/session.rs', 4274, '02c5b87f09cd50278df97fa5ebd36c9eccde2fed2e84eab4901fa8ecc30bf89a'), ('src/pxe/source.rs', 3620, 'bb7aa52f420283148643d04e06c5e5c37ddcf728ce3dd6d7a5ca5c15b099c14b'), ('src/ui/pxe_page.rs', 21356, 'dd9033e6ec5586374fec02ec493cecb4f881821d1b55aaae645e65e4db121e5e'), ('src/ui/smoke/pxe.rs', 20552, '001bcda3c167c093065c24a1f4799a449eeff059b69a89796f82e2436bcef6e7'), ('src/ui/tests/pxe.rs', 6630, 'ae03c79748ac3fc8c28e49d022e4590ece20a20cefabb800b8a130bb755715e7'), ('src/pack/retained_515_catalog.rs', 8145, '2e7f281414c82a5dc3299aea83e6fae92d3a02f40d1ba2029ca48866fdba88b6'), ('scenarios/pxe-lessons-v1.json', 45214, 'a2845fbd3c922d3dd5fba93ddb3df5cfcb335464ea52a51685bf10fd28f461fc'), ('scenarios/pxe-units-v1.json', 516, 'c9e18e27d826442093e4b623bed3d1a8fdec03dacde0986774a222509b723800'), ('scenarios/presentation/lpic2_pxe.json', 8889, 'a1c5b7a9061ea816cf3da6af9156e6a6e6854bd2eaac7869041277e70d0ca445'), ('scenarios/curriculum-continuation-v5.json', 289299, 'e4124dce797e982071a06578beaac5f6f1ffcf152d707edb84bfa88686dab667'))
CURRENT_INITRAMFS_FOUNDATION_INPUTS = (('src/initramfs.rs', 13087, '8e4f86c91dfe7c2742b58f06832895dd28753208df8b8561eeba9572fbd670f9'), ('src/initramfs_workspace.rs', 27926, '4f598355c293d58eebd8e2e3cad2f0c07f15007de43143639cb021e3a9b57164'), ('src/initramfs_source.rs', 3726, 'c9f7df6701c268df7f1c38cd0f7af76a5ed3633908ea40681df593d71814ff1c'), ('src/ui/initramfs_page.rs', 18140, 'b473bf45dc432facd8c7dbcbb7f70f49c3d9a28b9dc90f11c8d30389a2b92e2a'), ('src/ui/smoke/initramfs.rs', 21906, '414ffe55cfec339cee508d78484015bc684710525d7389c5a761cd1cfeecca6c'), ('src/ui/tests/initramfs.rs', 6926, '074f3f93e748a61a5b1c2b5bb7d14c69bd83cf87201c3efa1403dce8a9c65508'), ('src/ui/smoke/reopen/initramfs.rs', 9771, '10e630c2187e8ba6a95269faa4aaaa6e43b4a44363807af4c06cf788ab3c6e9b'), ('src/pack/retained_513_catalog.rs', 8145, '4fa8bf2c68674af550127c5f5f35600e4a3b8a0496bb05608ab68d746677be3d'), ('scenarios/initramfs-lessons-v1.json', 81578, 'd5c8d0fbfd883cb6c872be6814e091266e65ae56d4a36b45a3906eeb77ce5177'), ('scenarios/initramfs-units-v1.json', 581, 'a7359c3f3ecc437c6219264bfe50bc67587e798f9f60a56afce371ea9fc4fbb6'), ('scenarios/presentation/lpic2_initramfs.json', 11605, '7a0541a005c5112b4c27f03ec2654ef0b752a75ae99352615a85415fb1fb6ff9'), ('scenarios/curriculum-continuation-v4.json', 244132, 'fcd794009d632a7f0c729c8c3bb993d794e467a0312e318782fa367cdf2b1a60'))
CURRENT_DKMS_FOUNDATION_INPUTS = (('src/dkms.rs', 15504, '8d9c5b35d0f3c511f8665ac777d336f484aca4918e1fd2f9a5e5ce5d48e21cdb'), ('src/dkms_workspace.rs', 15563, '5fcdd7d6095f015f5e415e22605a9b0797a8bc08ed830d69fb94aa6e5a9b2727'), ('src/dkms_source.rs', 3420, 'd0b8921c422524df8c45950554a9065c64503903d9a19d3db4b46834f0ad32c9'), ('src/ui/dkms_page.rs', 13386, '5ee711dec5847f5509afe58deb1f8d7aa8bee9549e3422240e7339193ac58e6c'), ('src/ui/smoke/dkms.rs', 19463, 'cc177b76dfec5bba9e5cd595b19d0b98cfc5be4be8edfe6054f3a69669a02647'), ('src/ui/tests/dkms.rs', 6276, '57f1025e563cb4d986e3cc4e005aa7281c7e9b3fb45ed10659149786e4026926'), ('src/ui/smoke/reopen/dkms.rs', 9023, '3121ef8be18a28f420341b3bab9152d4ae2a09ca6908003272ab921cfbc08fe0'), ('src/pack/retained_511_catalog.rs', 8225, '76a18a9b0d1af87cbb080613efb4f74a9bddf029fa7a57bc74ae4b73f09753f8'), ('scenarios/dkms-lessons-v1.json', 22354, '6445fd36f3028c373e8a4ae9f4823ac126ba8cf8599e71b46ade695c6dceb1d9'), ('scenarios/dkms-units-v1.json', 529, '22e667aabad45789d5dec92c5159190d44f8714dff22140c6aeb18520c0eee4a'), ('scenarios/presentation/lpic2_dkms.json', 10068, '826c8d54e0fc96c03703d69b86f4fb9c00a254512d7bad21090518eef5c7fa6b'), ('scenarios/curriculum-continuation-v3.json', 162601, '4981635ef9860e97e7675a1d895af11fef78b17a1534114e745ec17e26304f02'))
CURRENT_KERNEL_BUILD_FOUNDATION_INPUTS = (('scenarios/curriculum-continuation-v2.json', 140294, 'b4fa8f29022b7331ba1ba0487d162ce72b896a31c2d306260318c6dcaf69fd86'), ('scenarios/kernel-build-lessons-v1.json', 45460, '9fa33df726afea2df5fc1f2dbe551b8b9386815fc813d2fb798b90e4b7066847'), ('scenarios/kernel-build-units-v1.json', 572, '44acea078b410aa870f0ac8f18845b47ed97759222f2375a3c62a7d091579da9'), ('scenarios/presentation/lpic2_kernel_build.json', 12661, '3df11dde154cc2707a3d6e5eb3b5e3a54e1c0dca2ed54846e0f7712de0205453'), ('src/kernel_build.rs', 18205, '2e59872c8a9c3ba4a074c318e325ac168a924d268cf9a12590d8f4a416eb5cc5'), ('src/kernel_commands.rs', 8248, 'a3cc657bb901ac2930f5454318a2233786c48622245ef12a4d363a70d95861f5'), ('src/kernel_source.rs', 3644, '6065e04067ecdccf1892481aabc7f03dc6658e399d599f4aad26692ed95ad09c'), ('src/kernel_stage.rs', 12224, '84548c48243f59009c58225e8b5b0dbe45c50dedfad14f8f45f2b81830e4bb4a'), ('src/kernel_workspace.rs', 30137, 'fa1aa6ce4e6c301b6c138d74124b8861cfd14e5ebce34b72158233a5048e8167'), ('src/pack/retained_509_catalog.rs', 8212, '25656bb8b7b42fcc40293d2dd0689c25a6c79d73a374fbed7db405039c919f0b'), ('src/ui/kernel_build_page.rs', 20691, '49e64cbf7b3a634bec0a4976f5fce3ade2008975f2aa9b4cd476f4a2b571c670'), ('src/ui/smoke/kernel_build.rs', 22193, '9b88683b481f705351fff77c6e7d8d894a5e59c6dfedd4a5589dabaa74420c61'), ('src/ui/smoke/reopen/kernel_build.rs', 10505, '493bdd7183dfe60597eaf38ed8ebcd85b6d48ef85b4c2110c30478e2f0fe6800'), ('src/ui/tests/kernel_build.rs', 9144, 'e2e5ffb86945fbf4e6a44520a65a87e08c25086a1f005c5d46583c987c530c29'))
CURRENT_POE_INPUTS = (('src/poe.rs', 6090, '74ff2820548c1289046bc5b8b5546399b01de1fbb859b529a6fba4312e09082e'), ('src/poe_task.rs', 4936, '61db7ef64a7544d17f4e19ca408d3c88e9d76002a80cd58da070372a6c7570b8'), ('src/poe_workspace.rs', 6408, 'd5678c38870bee83c967d5b5b7ebeab03c1ea8551ecb1b4db97569558d4a3e13'), ('src/poe_source.rs', 3190, '17268a04373596f574617dce96085c8b54ce361b87a3c996c3e894256e2fb8c6'), ('src/ui/poe_page.rs', 17816, '7bceaf148c413024044e6887768dcc168576f08a2d39b68aa0fa3f90c557f935'), ('src/pack/retained_507_catalog.rs', 8218, '9e455c14feff4044dcab09bb6967bdc0dbc8139e08398f4178795fa63c61f4d7'), ('src/ui/smoke/poe.rs', 23415, '655f530f60fbc71992d0e3a6633bf3c8dbefca0b698e8234e780f2b3e470435c'), ('src/ui/tests/poe.rs', 8870, '69bdcf01e74df15ba9f66c7ad446096ea67db2f9bad0dd69404ed9262f617945'), ('src/ui/smoke/reopen/poe.rs', 8503, '359ebf4b0c08ad3386df8630a7ae0be80a96670669260a6bb0f37feb12777360'), ('scenarios/poe-lessons-v1.json', 20710, '33c8024b467cd854fa6048cc1bada52ec72b7696331420658e3e3a17b59d4576'), ('scenarios/poe-units-v1.json', 515, '4aef78a70cf3d2c24f6f9f584a1df0e783f893d204f7c6576a1b3d66c522e647'), ('scenarios/curriculum-continuation-v1.json', 87899, '4e9c0f59a10c117bccc795ffb9e5f88efff44766e7f29fbf6e17c4d589148ed2'), ('scenarios/presentation/ccna_poe.json', 12565, '0245d41c61309cd3aafcceb275d38db8875181203af81f248987413cd649fa24'))
CURRENT_TELNET_TRANSPORT_INPUTS = (('src/telnet_client.rs', 7581, 'd0ecba972a49fe86246899fd6d8e6a320c1faa5e6f3b3cb97f76c88688444871'), ('scenarios/presentation/lpic2_telnet_transport.json', 17502, 'f16e1afbabd6af8e9b47fb02f8c1a46bb13d4d60798311eafb8f58c64be20137'))

CURRENT_NC_PROBE_INPUTS = (('src/nc_probe.rs', 6693, '3e91aa9c0fa986590252099713f3b98374e90d25848397cc3c9bc5d9dcda61e7'), ('scenarios/presentation/lpic2_port_probes.json', 15203, 'dc13b97486f64203beb1781c582dcd84753fb6ed2769701f328cca6a3fc9e700'))

CURRENT_IDS_RULE_INPUTS = (('src/ids_rule.rs', 15323, 'a9bbf608f00d55f31723f8a15298d7e727708f64edaad01e8d34c465749e977a'), ('src/ids_rule_task.rs', 7084, 'cad7eb4aa8d961dd5149f4f81427ec070e488b3c831a9d1dd4f6d707ef5afb37'), ('src/ids_rule_workspace.rs', 9827, '366df7afc0a9c12d7d7eea5d26d322f0071bdfdf3a356c4b1f4cc4968dbcef09'), ('src/ids_rule_source.rs', 3567, '613d7c3d69e4054647811fb1d00fedb53678349118e67fdf6deaaef26e80e7ab'), ('src/ui/ids_rule_page.rs', 16789, '1cad6a63adbc16ac3d624abf64ab585188665ba50c07ba837e675be24d87f32f'), ('src/ui/smoke/ids_rule.rs', 22685, '94a5fa60e0050f391e099ff28d1903c5164bd79b91407b280e96f47d44fd75c7'), ('src/ui/tests/ids_rule.rs', 9534, 'cdf3381956e274639076ce32a814f3904e95f0d91e3d431a593d5465331c32de'), ('src/pack/retained_505_catalog.rs', 8735, 'f9bbd0d89a1ff2a634a5215e89d9bc2eeeb92e1fbad25ef266e7f495b94cb350'), ('scenarios/ids-rule-lessons-v1.json', 41264, 'c0cd2589b372c4513ac7611790b442eedff6af37b5c4c07802c58987ffd29de9'), ('scenarios/authoring/ids-rule-v1.json', 41264, 'c0cd2589b372c4513ac7611790b442eedff6af37b5c4c07802c58987ffd29de9'), ('scenarios/ids-rule-units-v1.json', 535, '93eff7dba7be9556ec30888c52e2e1a2a51be5411d8bd8527929c1152c497d25'), ('scenarios/security-detection-lessons-v2.json', 74218, '692d95a5611a57c8e7333ce22bfa13473afdce381e4e9e44b035e1d229396f0a'), ('src/ui/smoke/reopen/ids_rule.rs', 9987, '1810c4cbbfd7332cc6debf4f408567e9c5610e7afb7063f1b93f2d52237d1745'))
CURRENT_LOG_FILTER_INPUTS = (('src/log_filter.rs', 12607, '24cc08368c6a93cf351d4fa0d66869c73fde35867a592c52f3616d11fd7469cf'), ('src/auth_filter_task.rs', 7544, '85019dc909425cab0b1a258f1dc952c86d4305ada9d41884efc34a5e6d58f623'), ('src/auth_filter_workspace.rs', 16017, 'a8b1730aa559529ebcdcce8af0779dc947160186d83457dc1fcac556a88e5bf4'), ('src/auth_filter_source.rs', 3584, 'b897e54dd07349b3529ab41e37d1e3f695df06d939a687f05ee0a7c7461669b9'), ('src/ui/auth_filter_page.rs', 19312, '68628a5bb669d299b34d7aef6cdc835747ae7bd3cfbfaf0bc40c0893eca969a1'), ('src/ui/smoke/auth_filter.rs', 23547, '7c1ca9ea4e84c9792161fbe664f889e6ac6316cedbde5f3feb9ee61e10ab8e17'), ('src/ui/tests/auth_filter.rs', 14040, '8e8e6606a0ecff4f41c8c9a05462c528cac88dc1b8fe6c2d664f5834fcfa9de9'), ('scenarios/auth-filter-lessons-v1.json', 33001, 'a774c01398c4280d58f86f74202f95f6e6ad5eda8b022a1329af930a84443caf'), ('scenarios/auth-filter-units-v1.json', 542, '44d90ae49299464a6dff6f134989c245313e816555d4748d27cdf33503f342d3'), ('scenarios/authoring/auth-filter-v1.json', 33001, 'a774c01398c4280d58f86f74202f95f6e6ad5eda8b022a1329af930a84443caf'), ('scenarios/ldap-administration-v3.json', 371375, '6ff690c3c5d65f6b21701272ee13a06c19b47fd0f5a5226548cd291863ffa922'), ('src/pack/retained_503_catalog.rs', 8214, '3a496c96993ff24f557b7df1e19920ccabadffef0f79779e793200a161b32b43'), ('src/ui/smoke/reopen/auth_filter.rs', 11132, 'df25d5961811afa0c77e3ec0c19a5c9565c1c698f39a3caaf17cdcc482bebfb3'))

CURRENT_RPM_CONFIGURATION_INPUTS = (('src/rpm_configuration.rs', 9280, '2362bfbe3f700dccb7479a524bec950abecbe3c5b35bbacd5b92037c1fe50ce7'), ('src/rpm_patch_policy.rs', 6201, '4bd1f0704c03495fd83c08c05a05a572e22ed035afdf4c314ffed24f4a7c21f9'), ('src/pack/retained_501_catalog.rs', 8216, '1d94f7d6e23098fcdf9f63874f89e6e3cf4fa2c11643676948ba8d3cf967c258'), ('src/ui/smoke/rpm_patch.rs', 20642, '0fed800ac99f582f26a7fed489d6c8f59e4f8863949711b691a6d139453ee3b1'), ('src/ui/smoke/reopen/rpm_patch.rs', 10660, '2ea217013948d010b7f9dcc458ccb6967177453909225ecba1102a8cdab6896f'), ('src/ui/tests/rpm_packages/patch.rs', 8561, '34b93cd861c8d0c5e36967346afa84b41bc5c97e056db4ed18a865ac4a6ea433'), ('scenarios/advisory-patch-lessons-v1.json', 83168, '58124d858cc632d6d77ce466121e77b92de17c60ca34bcdcb033515d63fb35ca'), ('scenarios/authoring/advisory-patch-v1.json', 83168, '58124d858cc632d6d77ce466121e77b92de17c60ca34bcdcb033515d63fb35ca'), ('scenarios/advisory-patch-units-v1.json', 508, '98276a9f131c14ecfc6418b49e060c746be0cbd67e683437d09553ea0852faf1'), ('scenarios/boot-entry-lessons-v63.json', 1989201, '63a961d9ccf4e42284deb50173a9eda917a16302b0bf24e6c4adb58988f155f8'))
CURRENT_VPN_POINT_INPUTS = (('src/vpn_point.rs', 22832, '0346df3f03a5389b549cf4b24793867450e8e7be6751931605e06368920d0e3c'), ('src/vpn_point_task.rs', 18566, 'f232dad261fa370dba8aa442f276645dbf11ea13512c009a95b0ce193c2bdf4e'), ('src/vpn_point_workspace.rs', 15898, 'ec48f072b661f5310a910c56b865f3e8432b1a0684b3df0b6bfe80cf7530a27a'), ('src/vpn_point_source.rs', 3442, 'c5a0139bb48bf48fbf17c76da47cc9d5bd3f49a221e505c6562b8a2dae4cc750'), ('src/ui/vpn_point_page.rs', 19037, '3753202503a29eaf3671b090612ccbb81cd82c46e1325cb399c32bef2daeb32d'), ('src/ui/smoke/vpn_point.rs', 22596, '403637cfbd9f55f3e40ffacee1283716e7e8030f8da49e2164fc1939d4055686'), ('src/ui/smoke/reopen/vpn_point.rs', 11856, '1e58a0cb536b260db244551f047c23bb10fde5c1d65643d4b52d6e9cc7afda8a'), ('src/ui/tests/vpn_point.rs', 16296, 'f111b794e2d43481d20231fe4ee95967f780904aa4ad6d24c9b0f818e67b7d7e'), ('src/pack/retained_499_catalog.rs', 8191, 'dbec6bfa61341a01de645e670a5922b0cbb96734441207dfb74c4f5a2d52d5a8'), ('scenarios/authoring/vpn-point-v1.json', 70715, 'ec5cbc6d6a73bc87a58d87bb5dae4e7bffae9bd019bb81377662ee0161ccb174'), ('scenarios/vpn-point-lessons-v1.json', 70715, 'ec5cbc6d6a73bc87a58d87bb5dae4e7bffae9bd019bb81377662ee0161ccb174'), ('scenarios/vpn-point-units-v1.json', 537, 'c852c0bcfa51f71aa1c11e1dedb5ad261e00d121e6684ab5e9f6eadaa720c51a'), ('scenarios/boot-entry-lessons-v62.json', 1944173, '8d3dc92eaeb6dd90f5af7235bc7552b97ad40c69bfc37f3f768684e5e9d61596'))
CURRENT_SERVICE_EXPOSURE_INPUTS = (('scenarios/authoring/service-exposure-v1.json', 143023, '95f3edbc7a937d857019b111f2fc2eb1de8adfd48bb959c2fb92fd6742acab92'), ('scenarios/boot-entry-lessons-v61.json', 1897072, 'e7d7b95f2fdd2791c79e7e021e467a953de50ad5c808611688bd99a595ffbb25'), ('scenarios/service-exposure-lessons-v1.json', 143023, '95f3edbc7a937d857019b111f2fc2eb1de8adfd48bb959c2fb92fd6742acab92'), ('scenarios/service-exposure-units-v1.json', 546, '25557b7a4abbbbd83221c1b5a4ed912e283cc619d54edeae0084abe40167176c'), ('src/pack/retained_497_catalog.rs', 8204, '7fc0fbff8e3a4c2f19ce90807534cd6a05031730386472191b2dc800a57e1a20'), ('src/service_exposure.rs', 14914, 'aad09ec0d2bb335e0e4d5865aeb7494ed717c747d753bee900334ae7ff94194b'), ('src/service_exposure_source.rs', 5689, 'f0c83295a91df0d64b42da81abca3b98fe27b1faffe6c96743be3fd098d82299'), ('src/service_exposure_task.rs', 8915, '1e0aaecf727d0fc9d94c496f31deeffc085a4f030057113a6d5a0d4f710290e0'), ('src/service_exposure_workspace.rs', 21576, '7ad1709d1144885ec7c2b65fb2a9c9615879da3419d4a2cd6beb9ee841651158'), ('src/ui/service_exposure_page.rs', 22128, 'd419545d7224ebb5b88b45baa011ed65274b87cc85731a41423d32293fd59800'), ('src/ui/smoke/reopen/service_exposure.rs', 13812, 'c6b647c41ecdd1076227f956518ced20ee81db0e0685312d77b01a63938a0847'), ('src/ui/smoke/service_exposure.rs', 30915, '7208c5b0ab82737fb148b4367bd7c4b3f069c6ae76a4f66ecd80720169d4f643'), ('src/ui/tests/service_exposure.rs', 15753, '1ad6f21e64ebbbc1cb6aad9efe42fe9c23f8ae6e6e63fb020e58816f6941c853'))

CURRENT_FTP_IDENTITY_INPUTS = (('src/ftp_identity_config.rs', 5422, '37400e9268f9d11bc38a3f8b5669a32912de8367d426e18edb20ee9d14fd011c'), ('src/ftp_identity.rs', 18893, '4418255d05e83186992245f054b12fc69ef642811ad614273459ac66a8b1e12c'), ('src/ftp_identity_task.rs', 21241, '55096ae39bb278a4fe76db537d0962d812c5205584647b0af6fd4ccb1a94bafd'), ('src/ftp_identity_workspace.rs', 23741, '276fefb06fdd5bae81e47865155b1f003e15c6b873abbd2902f0fa4f2ef5004e'), ('src/ftp_identity_source.rs', 4381, '22f6b0196514fd7bffbd3d61d08a086f54929f82b6e76dd7f58214d9345c850f'), ('src/ui/ftp_identity_page.rs', 25887, 'cf005e58ad822f461bab36c9bfd3c54998b79376db703ccb5fd0019a8bcf2903'), ('src/ui/tests/ftp_identity.rs', 11419, '8287c7a719c63dfedbe2299ce95b22b21cde149b1f0517bb705cc76314da93f3'), ('src/ui/smoke/ftp_identity.rs', 22047, '9694a17b12bdeb45b4c14cd53a090588f241e2c40c645a555117e37d99542dd1'), ('src/ui/smoke/reopen/ftp_identity.rs', 12579, 'fb6123357b7d9dc1ba2d6d3793a86bcabdb6b7d13a077fb63e720811c3891cc4'), ('src/pack/retained_494_catalog.rs', 8209, '7e37bdcdf46e20d19444f2f48e08cd3de9a2bb338aad5d28e668f10edd2bed78'), ('scenarios/authoring/ftp-identity-v1.json', 160708, '4030cc6614a45aa1344ef1b6ec4ed1ebf0f03fe0c9959e8c5c11a6d6f1e15a58'), ('scenarios/ftp-identity-lessons-v1.json', 160666, '46b575d44c158c79b20c90132484d1971c958dbebbfaa56722c5c7d09bb47a62'), ('scenarios/ftp-identity-units-v1.json', 542, 'a596c620d917c4cd6500e9709ffe5053b4325a677e74b89ae188b249a0d6e1a8'), ('scenarios/boot-entry-lessons-v60.json', 1820833, 'd03f6af4308c0593a65fafba27a8f4530ec6c113d8f1a502afd9252c55bb30d8'))

CURRENT_LINUX_V6_STARTUP_INPUTS = (('src/linux_v6_saved_filter.rs', 6166, 'b2b9dd07b67aa923497d37256c78a9d515f8900a7a0a4f07b01c2153e470ce87'), ('src/linux_v6_startup.rs', 9660, '53da6791d4d251ee4516101581c40f2c8ab84ea358e8654519335ec90c580093'), ('src/linux_v6_startup_task.rs', 10900, 'ba10e5e1035aa5cdd665fa901d22cc609bcd1db2ae97fe17d2b42df664f14324'), ('src/linux_v6_startup_workspace.rs', 14735, 'b4d24799440565fb7eeabe448cfa319a852b7aceed29a49213745cf9251c4907'), ('src/linux_v6_startup_source.rs', 6274, '6a0191b26a9fc2c82cae057bd85b072fca6d0db72e7b9342b0eb3bd2b6567d70'), ('src/ui/linux_v6_startup_page.rs', 22345, '64a60ef9f57424e620e25462be0518b7f097cb6ec59a3a405c6dc7761c406c77'), ('src/ui/smoke/linux_v6_startup.rs', 21808, '313f187f9c0d3914a60b493b54197e6a6e386ae295962a90c56f5be7eebe91f1'), ('src/ui/tests/linux_v6_startup.rs', 9752, 'd730d86f7f221349ae7519b3ceaccd9c93d03bca77b8ded5bcb3f2eb09e1793a'), ('src/ui/smoke/reopen/linux_v6_startup.rs', 10474, 'a5dbd74310b2e7b886180bb037308d16716766b9dd52a2ebfe4325d3220365d0'), ('scenarios/linux-v6-startup-lessons-v1.json', 89471, '271123a002673fa61a1bd32ee1da6e6d611b0dfa6f8e1f475f998657578cd488'), ('scenarios/linux-v6-startup-units-v1.json', 502, 'a56ce8264e915b0ffe6cf3d789f1b36ddbbe0e5fcced6d310e4e4fa2d1cf9c1e'), ('scenarios/boot-entry-lessons-v59.json', 1745809, '62186e874573d3de4edad09ec1f142c39acf3cdee04ec67a563ac818a05d0cba'))

CURRENT_LINUX_V6_ADMIN_INPUTS = (('src/linux_v6_settings.rs', 8553, 'cee05332184ccf0d1da7cb3f3904529384e2caad532ad85b39632070f24e7c9c'), ('src/linux_v6_route_equivalence.rs', 2327, '4995c51d94c38b1f31651179f2001cfb46da7412908743e97714426912aa16b1'), ('src/linux_v6_admin.rs', 13813, '7254a420534d78197c36ef18a0a87bcb106c1e2398f6af63fec47ed8b22a47ab'), ('src/linux_v6_admin_workspace.rs', 33204, '060ddb01da270faf6b9a3d6103fd7efa05d30500a5510bdc0e217d07d11d4219'), ('src/linux_v6_saved_settings.rs', 3892, '4ed6d805a456343d8b398df2a07b592a70db49030ba0c683c3817663b38e3acb'), ('src/linux_v6_admin_source.rs', 6759, '5abbfa6ff105b64bda43e8fd373949bc814e0fd835441a43ff97b258cfb33bd4'), ('src/ui/linux_v6_admin_page.rs', 26814, '735e4b3e7616feba5fea2128bd149bb14247a36df033a6b789395daeba40ff3e'), ('src/ui/smoke/linux_v6_admin.rs', 20863, '1e9d26b6d074b5cf51d63013e3918eb1ce971320673bbbe1d2210df981300608'), ('src/ui/tests/linux_v6_admin.rs', 7563, 'fb448f47a0c69dbd9b6dd246257067ee7282920777474331b920d899c9a381c9'), ('src/ui/smoke/reopen/linux_v6_admin.rs', 10071, '647ecbb7c0b690670104b88710d79ffc267f9482eb522c6f5e7c3c2547ccea3c'), ('scenarios/linux-v6-admin-lessons-v1.json', 141322, '4e11186903864e3ad5915b5f6565f4cb37907fffa09d744e504d5737cf839629'), ('scenarios/linux-v6-admin-units-v1.json', 608, '21f447f5344434083d9aa82aaab6cf75891a67fbf2f0bc9749ecc3b40489ca10'), ('scenarios/boot-entry-lessons-v58.json', 1707767, '2a6c8a4c4e12cd742afcc8f3e046cb0f4cea369fc14e343a6a573a9e011e4fce'))

CURRENT_LINUX_SERVICE_RULE_INPUTS = (('src/linux_named_ports.rs', 14384, '8e713f1b1a488474adb6bec2d7f3288305ff4d1f7e85b57a86faa6004fb510ba'), ('src/linux_service_router.rs', 14771, '4160fefd73f65353dd6bd169d45efe51ae19412ad674ae16ab18c9eccbf49ed6'), ('src/linux_service_router_source.rs', 5413, '97613b8d2420508cb3a0e1a7184245486e2aebe1bf05974f792bd25f585a47fb'), ('src/ui/linux_service_router_page.rs', 16368, '4a79c1e8e74ebbb7dc94a62febc8591cdceafb175a3724e3c0ccdfd3ee039383'), ('src/ui/tests/linux_service_router.rs', 12031, '1bacf8730286028477674aa2fa2df89830a0008db836b13bf1f9f9ebb52170af'), ('src/ui/smoke/linux_service_router.rs', 27667, '6332642fa9587eb891ec1127c78f6d47c908d4e4e1a1cb11fff0490560ceef7f'), ('src/ui/smoke/reopen/linux_service_router.rs', 11150, '0ecb8df10a21b1a38f2944ead8f274443fa511c260cf1409f176caba50e7515b'), ('scenarios/boot-entry-lessons-v57.json', 1642432, 'adc95a48fd79250cece25c1edfc3018b8c78454617c49be1e70e8da0ba960a27'), ('scenarios/linux-services-router-lessons-v1.json', 80264, '153b5ab087761d7dd592552730ea84dd99bfe24bc17d6e86f4c2f269c35163ce'), ('scenarios/linux-services-router-units-v1.json', 485, '1c05d1c39ae86677aa178b3f193565c62821e1191db0f13cd807101d7f5fe432'))

CURRENT_LINUX_IPV6_INPUTS = (('src/linux_v6_regions.rs', 17158, '8666f3cacd2fc1d3f64d99aa505000a6bd70f3f2f6ebdeb09ee5faa5d8eeee67'), ('src/linux_v6_filter.rs', 22850, '450e116acb9623fc9517bf7aaec4976140df4515a180a1cd310ccb9dd3389c9b'), ('src/linux_v6_policy.rs', 8496, '1ee6393ff4044cdc69ffa336f78c87d4ba1302def36ae5a96cd483dbd9b16b84'), ('src/linux_v6_connections.rs', 6076, 'ef84c8082eb5912d7600e36447c7edc228eab6c271df1c85e2a449b4b4f20e75'), ('src/linux_v6_state_filter.rs', 14610, '2897624137d36b4bfd55a04c63912de3fdca2dea94957c3e2be90d9314f1dccc'), ('src/linux_v6_path.rs', 11678, '6f5bbab386f43078d2c9b395cc0e3ed45dbb49a85c2300af61e20cb843a6eaa5'), ('src/linux_v6_flow.rs', 4969, '11a29f0af3ce6166d98bd678217c2aa98088ad8f74918e26900f75f85749eb06'), ('src/linux_v6_workspace.rs', 26528, 'dc4fb355f5dd1d43169fa6cf9735352420edbd8062b6d3ccb78804364348c3d6'), ('src/linux_v6_source.rs', 4877, '5f3b16982bb4cbd66c1cf01de243eb7ee8aeb04a8072dcf1910ee3bbbb76d2ef'), ('src/ui/linux_v6_page.rs', 23487, 'a4e7978abec9dceb7f05bf87ff6d507bceacc71165c9d46a75e51f2d27b8370b'), ('src/ui/tests/linux_v6.rs', 7841, 'ab4e882c8c233134c1f107db864146e1a680df8f962090949c50bbf6fb46032d'), ('scenarios/boot-entry-lessons-v56.json', 1607227, 'a809685f948bec8d6456b28b3abcf40cce8e4b797e8c9cdc5aa52928960715bb'), ('scenarios/linux-v6-lessons-v1.json', 91965, '8e5b359630e91876b14d6b5d21168f3ddc9e20d8d66223a4efcaa3e1e3ba3472'), ('scenarios/linux-v6-units-v1.json', 528, '96a2ad28483de85b457848f874bacd572bcde9470d590a77b0c880664000d6b1'), ('src/ui/smoke/linux_v6.rs', 26072, '38ec4197b2fc56778abd13d706113c880aca82dd93906a37942864cc7e79b49d'), ('src/ui/smoke/reopen/linux_v6.rs', 9226, '2106208d47e3cb9a9afb79cc22f1a13a81954d7d32b87843e8ff13ac70f69093'))

CURRENT_LINUX_CONNECTION_INPUTS = (('src/linux_connections.rs', 9090, '9c5b3d77dfdb77e92077c80590dcf2475ba0399f910cd8e50678f9aba0318fb8'), ('src/linux_state_filter.rs', 12551, 'f6fe194b3140de4efe5e2f69ba50b9b70c9b8ba190b5cf334653f574b35b9680'), ('src/linux_flow_nat.rs', 9215, 'a91848dfb7ce3fa65797b3cc0ca040b665b456d73d347f8ff938df5225776f1f'), ('src/linux_connection_flow.rs', 8739, '7463126740625b41b08cdd173237e67967c5e1aefdb0851315932c62a66cfe60'), ('src/linux_nat_commands.rs', 15109, 'c07606f3a0cd69b857a3da3e1cb58722b856dcad519578aae66fdacfea015788'), ('src/linux_nat_policy.rs', 11869, '8586da443499c0668679d935a744eb89eed8fe933e5673cc17a04ec2cba9bd8c'), ('src/linux_flow_workspace.rs', 31309, 'f573c0a1b6de9b09fa8491e4dbb910e8c84836fd2564d442916c85772102f4d7'), ('src/linux_flow_source.rs', 5054, 'ab3ca409ea11e16370b28925cdee71d71c8f0df22c37d18132e2127b48f29f4f'), ('src/ui/linux_flow_page.rs', 23171, '7ecfa7042f6162f19eea94caac61d69982a2cab9efd1a6031da1bb3924daf9d4'), ('scenarios/boot-entry-lessons-v55.json', 1563392, 'c40c40cf5a7b9bd0e51f1a9912df701c248b8973ebca07042a69bc2e0b8c2992'), ('scenarios/linux-flow-lessons-v1.json', 186404, '793bc2638f9151a0368559906a62088a7b344667dea36f0a26e07e2491376b4a'), ('scenarios/linux-flow-units-v1.json', 477, '72c5ffe401ba8243fdb8d8303a61bcbb3f7c7c3d979f33368ba4d3d5d1555034'), ('src/ui/tests/linux_flow.rs', 7950, 'f60781f0398e3627deaf334a19df4a79b663fb32d5b159e16837f3bfcf3dc5e5'), ('src/ui/smoke/linux_flow.rs', 27334, '6ad83a9c3c025c7a651588dc151cc124288d3c5e2cd921bbc3e5048ea7b45f40'), ('src/ui/smoke/reopen/linux_flow.rs', 9194, 'a4934551a78a07788b111d08f4af872979bd6198b1412d270b49e4304f7d9cfa'))

CURRENT_LINUX_ROUTER_INPUTS = (('src/linux_filter.rs', 24420, 'a5998f3c3660390bf923520c5aa9dded7bb6606de835972ef6f39cf42e868748'), ('src/linux_router_path.rs', 10098, '89cbcc9e2b5215fa136dcc0e6633fef6676518515aa2009d731534d3e0ff330c'), ('src/linux_filter_regions.rs', 12379, 'cfef8cbf7874ef21eec094a9fce93550250f246d46aa0a39ce30fdf475da051b'), ('src/linux_filter_policy.rs', 7552, '499277b362064dec0a4765c0cff72ad1fa4b976678ab840957444ae074a7125e'), ('src/linux_router_admin.rs', 18441, '9eeca34ab34425798e0c8b36375e5352566c4aa87587fb499514475f6984e791'), ('src/linux_router_workspace.rs', 24132, '53204302c005b4e048b43897aa9653564a8a58848c5f53e6017fd1a953f1ee39'), ('src/linux_router_source.rs', 4403, 'e2f3f488e48800d8c06319e1973c4459827646cc4fe98bd2b5031ea9f63740b7'), ('src/ui/linux_router_page.rs', 22490, 'dfc934013d58e12ef4018d78b7dc9f7753c5052b146064a4cc5c0b537ede3859'), ('src/ui/smoke/linux_router.rs', 22722, '3a467c6560962af0b84f809a6fb204e8b1b8c6ae14f6cf4c78862b6526456233'), ('src/ui/tests/linux_router.rs', 11032, 'e2f64c2099afdfd2ddd53386c6db4bacebf412d964135fae9be31ea655c1cee3'), ('scenarios/boot-entry-lessons-v54.json', 1484144, 'bb114a1fcc7d51ebceaeb4c89c5900def7dd21c3a391449b6351837c72767e23'), ('scenarios/linux-router-lessons-v1.json', 156374, 'd4d71c3d79228e65aed3676b2c621f0727157e216d10c669a95228e99ea07fc3'), ('scenarios/linux-router-units-v1.json', 586, 'ff511bf89789be9d05d2173e10777a8b4d4e02a9a6d9db5c62aef277e284b1eb'), ('src/ui/smoke/reopen/linux_router.rs', 9444, '5cd95f8befadcc44987551d548d8caca167b054d2fce2cf1156270a8371e4477'))

CURRENT_MAILBOX_ADMIN_INPUTS = (('src/mailbox_admin.rs', 18536, '26d0fa675fee9a6355cb82a38884cb68c37a6bfaa0b4e4927c44c78dbc5e0377'), ('src/ui/tests/mailbox_admin.rs', 8396, '4aac1ba01b9280bbeabae7bd0053c924f920cbbcb87e98986d891cb8219a81f8'), ('src/ui/smoke/mailbox_admin.rs', 6391, '7f247a8f09a3c871f03caf39923e4883efa22caa3ee1b1f950e860e204f5f968'), ('src/ui/mailbox_page/admin.rs', 5478, 'c13120852d04b2b79bd8251f075285f7851c7accc8bacd6492a3d5fd2e39f6cd'), ('src/ui/smoke/reopen/mailbox_admin.rs', 7840, '4792484b80bd3865e42663ee4886efe66ed5320021cfc62657421924f76ada6c'), ('scenarios/mailbox-administration-lessons-v1.json', 46373, '2965c1ea7fcb6a8541795460f1f8fe2b4cb45441c84373b28ab09a82b5cc488c'), ('scenarios/mailbox-administration-units-v1.json', 427, 'f36894eda769cd250b0e722f894d17d644d0af8b6c6b9b8ec00517892c65f28b'), ('scenarios/boot-entry-lessons-v53.json', 1476606, '952afbf08dac202d6670d7eb04735728fc6d15a85ca25799ba394a8a2011969d'))

CURRENT_MAILBOX_ACCESS_INPUTS = (('src/mailbox_config.rs', 24254, '0c5ae5bd46b0dd9dc64af2b762112e8c4ee9527c243f17834b7a42ffb584589e'), ('src/mailbox_access.rs', 24688, 'b6ba8ce89b804c6c8372dab57197ae01fe164d340540a72a3ecc470330720bc7'), ('src/mailbox_integrity.rs', 7792, 'db13506e03daf3b5c3c39c0102ca48849a96884e21e7902f11f285d7330227bc'), ('src/mailbox_protocol.rs', 17297, '473b12fe729fbf41a704258e28ad57de8413f869acb32348e335e884a2a5ee86'), ('src/mailbox_protocol/imap.rs', 21297, 'ce1a99dfba99df63b527db0bd804e086272a1b4d96ea66bf8c14a41dca8d6067'), ('src/mailbox_workspace.rs', 28256, 'e315cfb2b5e9844cc13f033f6435ed4a32e2b439a6fab878e645b9217576a2fb'), ('src/mailbox_source.rs', 4401, '5161fa5f69f26eef05973a9ac2c67f1422d158ad805902ff7b26459a48d28477'), ('src/ui/mailbox_page.rs', 27051, '253b4a85ae4fcf2dd68fbbf3632a4d6b8d15ad59dd83de2169f1efccd76a0c23'), ('src/ui/tests/mailbox.rs', 9222, '8c68aaa0282affa3251bc55286b5524ae08db6af71219b64a3d8afd007543b5c'), ('src/ui/smoke/mailbox.rs', 26335, 'bee1d6aef70caa03669465591993afe6e4034bbbe2c99f9424a3b5bf632f2a7f'), ('src/ui/smoke/reopen/mailbox.rs', 12168, '697c2bd6742c7997bcdaab900ccf5497af659a4a838b9e414d90ca6d7ed0f150'), ('src/pack_retained_mail.rs', 54930, 'aa210035132f87135a5175a355b3b1c25564e957a80ca7d71905067f0ecdfeb2'), ('scenarios/mailbox-access-lessons-v1.json', 102033, '75032e8a3de04ceed1029d563d1fe17d9c371342f81fd268b6dea96730dbb1b7'), ('scenarios/mailbox-access-units-v1.json', 563, '6014208620ca9ad7f6240b1726ff187d8f1a062c676e03ec6b1dbeab3fc52347'), ('scenarios/boot-entry-lessons-v52.json', 1430281, '679468d12583a43128dd2a921c1502e29730656f5d43a77d37df9ae6c87f8eba'))

CURRENT_PROCMAIL_MIGRATION_INPUTS = (('src/ui/smoke/procmail_migration.rs', 5402, '181f7f53138105a2624c8367047e6278cf9691e9f08ff95915374cf6e012b109'), ('scenarios/procmail-migration-lessons-v1.json', 17804, '06843724b296501998928268f5d40b126da21e9a0942f5f31926da08f5242d15'), ('scenarios/procmail-migration-units-v1.json', 414, '3ee1ab1d771fdbce908c7f25a7e6ba2b260802ae2dc8aafad5df07d6ba9fea8d'), ('scenarios/boot-entry-lessons-v51.json', 1333086, '471110ad4c3223716dfdd97662a33424b643f3eec89fabcc41ac758994e2c87c'))

CURRENT_SIEVE_VACATION_INPUTS = (('src/sieve_vacation_history.rs', 4794, '920685b0618ca67b5fa9de233888fc24d08760d1af37413c0fc3928b0572a001'), ('src/sieve_vacation_policy.rs', 13575, '6064dabd20c85324244b9837ee24bf244610e0c52e0b83b1ae0ac6a5c6283034'), ('src/pack_retained.rs', 15831, '4fc300967370e6200c67c7598c180bde3ef95ee482a0bed3e194af2f99449c4c'), ('src/ui/smoke/sieve_vacation.rs', 6035, 'cccc1d490e5829969be9e80e2857ef5718c1a7c1d4daa85d1e0590a8b7e61d41'), ('src/ui/smoke/reopen/sieve_vacation.rs', 9332, 'ca66db2fb0b099c769caff55c52a3f5e127802342292a95d6f24781b33fb733d'), ('scenarios/sieve-vacation-lessons-v1.json', 55570, '08d6e0efd317eef69ccd27ac309851cd8173d243c629c56235f7c983f008c081'), ('scenarios/sieve-vacation-units-v1.json', 482, 'a35c4c47786fa372dd2562c4daee4ea4ef4a348b1ad4b9b9b5653c45ed56b288'), ('scenarios/boot-entry-lessons-v50.json', 1322857, '247e977a9970097f2c353e86ffe20ba40421b3484b667ea5298ed0310218b165'))

CURRENT_SIEVE_DELIVERY_INPUTS = (('src/sieve_envelope.rs', 2995, 'ef0ef6970f4677e44a001759e385db6b15a855bf8d9622a0b39ab9fcfdb2dbac'), ('src/sieve_delivery_policy.rs', 2165, '7f91e258636b0e622c9919786dc3f1a89eaf9645604793a8742441a06e764210'), ('src/ui/smoke/sieve_delivery.rs', 5304, 'f7a248c6cebcca0115d78e6aa6707bb384da374c269cf750b1fbf20dfc2e5536'), ('src/ui/smoke/reopen/sieve_delivery.rs', 8665, 'a4dabb4f7e1292f7be9b50bca1bc897952e344a0fa72aee8c169994ced58d5ad'), ('scenarios/sieve-delivery-lessons-v1.json', 54593, '13ba3e74c6763952a04df682ac3e02696cba45111a04c43421f49356136bd164'), ('scenarios/sieve-delivery-units-v1.json', 529, 'ecec43f6a898ada32d42f4d131ca894018ec78c6bd576c8aeb03c611911a0d08'), ('scenarios/boot-entry-lessons-v49.json', 1293202, 'e22cd2a7e24f0d2cd0261547e392e4dc94ec569220fc384a180dbc0d47924661'))

CURRENT_SIEVE_ADDRESS_INPUTS = (('src/sieve_address.rs', 8140, 'ed0291dd06499ca974cbfa3a02133583bb94d02611cab9704ff956fd6cebeb61'), ('src/ui/smoke/sieve_addresses.rs', 5108, 'f04c5f99e8dd98db7b4c8b6b2dbdd89ca52a988b5a6c88ded7339580d93cf66b'), ('src/ui/smoke/reopen/sieve_addresses.rs', 8692, '93f7b334845e8ce26d386726736ff230a5bdf02f18ae4a6d7c83204c5cec943b'), ('scenarios/sieve-address-lessons-v1.json', 29540, 'c9e4fc95e055e976934768549f7493ac2ed192fa32332eb791f42596aad435f4'), ('scenarios/sieve-address-units-v1.json', 448, 'ad30cca52dd22c8a8aec394d44266db796c181b80af9c60d6a7e0d4c59023c4c'), ('scenarios/boot-entry-lessons-v48.json', 1261597, '8dd7b2af766b854aa92d7e76478b9cc2a61014b7842947458b70cba937b52f2e'))

CURRENT_SIEVE_INPUTS = (('src/sieve_lex.rs', 8458, '0c6fbf84578b1f0e2d1770bb92676cda4d89a0b7812024993e8fe7c567e765e2'), ('src/sieve_parse.rs', 17120, '70f92c8dba0ade98dd9e1e4582d72d2db0603092b4693425abf0c6037fac37dc'), ('src/sieve_eval.rs', 28238, 'ce4f2a32dddbdda6b78a24a3f1a82e1176292a128b7252234a811056638673e8'), ('src/sieve_workspace.rs', 36212, 'fd6203b5c4ea6fb1a5bd23232c566b4578c00a7b8d2a1aa3b2460d7cdd631dd3'), ('src/sieve_source.rs', 3308, 'f3bf639d8683e8e74550d39bb02d90368c89c4472791095422110e8d9d74a42b'), ('src/ui/sieve_page.rs', 22736, '6fe68fb0e1dfbe4247e3557305ceee8400b2f203deddb0b0888d9b5ab3d98161'), ('src/ui/tests/sieve.rs', 21532, 'a0c5869d9ff03b6b8f27da6dd6d0bfe0a63243020159175c070dd316b4b1120b'), ('src/ui/smoke/sieve.rs', 25051, 'd85d3fe474d09e7b31341cb386f72c1335106e057b18c55d9a4d66539d5591a9'), ('src/ui/smoke/reopen/sieve.rs', 8589, '76415bf3cc6fb7da153cdd8e01544ab2712a077b7d23aa19683638998e57c41b'), ('scenarios/sieve-filter-lessons-v1.json', 46569, 'b2f4f266ef72ceb03e73e3bf5d611f4e1b2c8d7c3f66c68c4be4747b499f8c26'), ('scenarios/sieve-filter-units-v1.json', 489, 'fcf14daf25b20c812aaed732e090c5d42cc7949f018c7fde6b4e957ad59f2666'), ('scenarios/boot-entry-lessons-v47.json', 1243238, '2649f6154cff774f5ef2f830986ba95456c70b3fc23d4e1646aa9057d5f59699'))

CURRENT_SMTP_INPUTS = (('src/smtp_config.rs', 15299, '8ee460cc06db80529abf66e708cf53403f5b37c3e32a9579bf8f52f103de1b74'), ('src/smtp_routing.rs', 12989, '694176479f65efd9c9f063f3609316901de02dca3035a87ed6cfc3d6b263eef2'), ('src/smtp_queue.rs', 11967, '9ec8f324c87b62f406fb908c372da9f1ec018a708a39e541f63edeae9c7807fa'), ('src/smtp_equivalence.rs', 1658, '3676910c6b5254f3bb67dd7b4b93933217f4a0beced8a9f24b547e246b4691f8'), ('src/smtp_workspace.rs', 23225, '9fa1737bbf0d737156bb4882e6782aa308ade99158f3524ec3d2065019722950'), ('src/smtp_console.rs', 6831, '3bd587e1f570d18df6116b66f5235ef21d2ac57464553c922e2e3818322accd1'), ('src/smtp_source.rs', 3063, 'bd24019281b1051d59f0c834f7aa7a72bc1f79ad797ecaaaeea20251b79fed7a'), ('src/ui/smtp_page.rs', 24645, 'e4135566c201e99564104e71471443df3de211d7083ac502ef6400cb950f0863'), ('src/ui/tests/smtp.rs', 8279, 'dbb795cdfea18b6318818de0b07d23d1364ef9dd0a4a42889d7960ac1952cda1'), ('src/ui/smoke/smtp.rs', 20760, '13b8c8b42102bf67b72e350d2069f4393a3b942936882704262a6f739be34b96'), ('src/ui/smoke/reopen/smtp.rs', 9805, '25403b7d4c6f16aebc000cce1b7a9cc8b6310fa599ce2e187ce8a8229a3ecd2d'), ('scenarios/smtp-routing-lessons-v1.json', 111797, 'd1da72170e44c37e07d9ce69c4c8eb9ac2c5c12f48f385fcb6a64947d4f01d06'), ('scenarios/smtp-routing-units-v1.json', 562, '3bb16037b9a1038381a09279504029a1cb0ed90cde99ffe4aac4c9a3981a82e8'), ('scenarios/boot-entry-lessons-v46.json', 1211408, '9f7a125ae52d8098730f2cd7af1bbf0eceed2cc4c8923855599269627e6fb57e'))

CURRENT_GUEST_IMAGE_INPUTS = (('src/guest_driver.rs', 8911, '18c6fb3773e83097c74fbdd2d95d64ef21cd4f36f414d9acca5d4079e82dcc78'), ('src/guest_driver_workspace.rs', 21180, '328a0af6407afea491e79b03d37728900ede3d2377151db28ec7b94af3ebb24b'), ('src/guest_driver_source.rs', 3189, '3943dec20cc64cf4c7712626ed3cf78f820ea287c3569942db765ce585d86a9c'), ('src/ui/guest_driver_page.rs', 27651, '443a09560f838791aa436686a6ac26da4ddc808eb3025fff761a3cf393ff9b6d'), ('src/ui/tests/guest_driver.rs', 9404, '0d1fa4c5492a76b74f11c80e26336653d37c2e3b080e0d07ff25f63ed9cff2ac'), ('src/ui/smoke/guest_driver.rs', 20021, 'e1d1b18481c85eca9f423dbcf51ee678e1288cb42d8e998dad81de0a34ed7e41'), ('src/ui/smoke/reopen/guest_driver.rs', 8935, '72318a729b2d0d3990d0b4667a04ac92aaed96fec7b115f3a7fd5bd3eef5f643'), ('scenarios/guest-driver-lessons-v1.json', 122474, 'f28210f80a5c6e61b94d633c7d21e61226ec86cf0927ec6f88f9c2b464870434'), ('scenarios/guest-driver-units-v1.json', 556, 'fe8896eaa9526129474a8a2b2836ab8b639e4fdc71bc52d6fe4d84aa47930da2'), ('scenarios/boot-entry-lessons-v45.json', 1132835, '6bfeee866d382e9465165c2bee9bb7c3fec42fa5e7c0f95d0d0c41929189a993'))

CURRENT_VIRTUAL_ISOLATION_INPUTS = (('src/virtual_workloads.rs', 8183, '98274537fa0622d3d8ed20b3aca49e37a44d003a6024a0813da55ccaaa237514'), ('src/virtual_routing.rs', 9175, '0a74339809ed0829ab9bad793b11de83792b0c459a7ed7762b3f0a11bbdfc324'), ('src/virtual_isolation_workspace.rs', 18010, '37fd8d0beff0ae00abfb28d3db7438d27b2c218dc3e10a8db05b1b978a37398b'), ('src/virtual_isolation_source.rs', 4907, 'b412ff1d584e36cff5ba0d3e17459651b64e05ed097e185e4d7adbcd6e77508c'), ('src/ui/virtual_isolation_page.rs', 22384, '33cc30da535db9d674f0b63fc799dc16dcc0742da59f78c1af540053f8306d21'), ('src/ui/smoke/virtual_isolation.rs', 25623, 'afa3150f01ce7a3b88a617c8577c6b9b16506b00041716da6796829eb5ae968e'), ('src/ui/tests/virtual_isolation.rs', 6446, '1cfed1d51e4ba4f38637b1aadd2bf49bc6d8faf5a69628264f0649e4c5310f1c'), ('src/ui/smoke/reopen/virtual_isolation.rs', 11211, '965431f3cc42b05c73ca955e6ba610e6d2daf93168be0a6ee4e768a2416ab745'), ('scenarios/virtual-isolation-lessons-v1.json', 56343, 'e662a6537ef05fcad1b3e0d36f6e7abd637ba3b7da1b440b88ba1d282422b407'), ('scenarios/virtual-isolation-units-v1.json', 637, 'a55658892b2bcd2fafde4ebc7ca17bb580a0b67a655e0538561138c7d7b1f294'), ('scenarios/boot-entry-lessons-v44.json', 1069108, 'f6c7c7fe1afa8299091bf6d3213e6327ac4471d0fccc7abd250002f0dfe5c12a'))

CURRENT_SAMBA_SHARE_INPUTS = (('src/samba_share_config.rs', 9621, 'bdedc74f5343cb6e66f4f373a75d1cc3921c8f3fdebf80c7fc73281cdd69b951'), ('src/samba_share.rs', 18245, 'e68cf0ab03715fb2e85eb8074d3c53045fd98dea11ee7d474c5b08eb1fa2982c'), ('src/samba_share_workspace.rs', 29938, '3a41bacf85f07722d78b17320120c0187a5d8c0ddea383043653699640d0257b'), ('src/samba_share_source.rs', 4981, '40e9d15e1ca9ee9424e832d85c8789ae8d7ca962873a367390f03b9253b67bf9'), ('src/ui/samba_share_page.rs', 25132, '7ef6de5d05aceef9656598ec9c3dad4d04e3add7ea6556ae991c78613d041247'), ('src/ui/smoke/samba_shares.rs', 22790, 'fedaa3bfc6260b0ffa05f60407e2d4bbd53b194c2c3dc7f13e4aaf02418c7b02'), ('src/ui/tests/samba_shares.rs', 11897, '535151dc42311800ffd79e879cd2fd007aca3de88a83c66d220734c79109d72b'), ('src/ui/smoke/reopen/samba.rs', 10418, 'f6f625a58ddfec2af1a18ed570e90864a765a84809492d26faa0d1a79fc51b29'), ('scenarios/authoring/samba-standalone-v1.json', 193120, '20d22a2acfe14dc4c3a181542c519623d1ece69d362f455a34a5daec29744e9b'), ('scenarios/samba-standalone-lessons-v1.json', 193120, '20d22a2acfe14dc4c3a181542c519623d1ece69d362f455a34a5daec29744e9b'), ('scenarios/samba-standalone-units-v1.json', 579, '9fae60c296e05ded8d9fefade4d0a28e7b30cc5c414af84ce0da5079fab508aa'), ('scenarios/boot-entry-lessons-v43.json', 1042191, 'f906e528c5a86ca7edb37940ee43537fb6574b554b6137ee9012b915e2250200'))

CURRENT_NETWORK_TRANSFER_INPUTS = (('scenarios/authoring/network-transfers-v1.json', 283877, '3c0bb3da99de28d910f1b31b5396c831384b4c65cedd762c9a5dd39e10726cbb'), ('scenarios/boot-entry-lessons-v40.json', 1823044, '2bd4fae48f8fe27b2107196a854e724a168125082db395d1da86465bb6f32762'), ('scenarios/network-transfers-lessons-v1.json', 284888, '51ab26f529d2af4d6178405c2b2e2abc5942511ff927031f15f48b83e882a067'), ('scenarios/network-transfers-units-v1.json', 529, '2f2ae11f3726af6705afb114057d043ca89935b9449ca5c9b5f82d03179ee335'), ('src/network_transfer.rs', 24488, 'c49941d9f8edc93c24cd53cc5aeec526d2473d2ff921860d478feb98ab49e1bb'), ('src/network_transfer_source.rs', 3692, '59ff1598727f832e7d8cee03b214ff4caec43894ab1471e4e3d21c59b825ff91'), ('src/network_transfer_workspace.rs', 24114, '22136e4c0ebcc0c712d90a22d05df58c019020dddfd8fc1f6e7f6b83df873718'), ('src/ui/network_transfer_page.rs', 25660, '0a73d3a8d71dcf0c06bb68f8cc0f7a2acf105e85d3e3f6d44f40ef88234e0c5e'), ('src/ui/smoke/network_transfers.rs', 25467, '8d0c62c307374460d07088e5f9d32a697f9ea20b31ceddb1139fc587291ed2a7'), ('src/ui/tests/network_transfers.rs', 21678, '372d328d0c6459aa8ab530ed701753c9c02b433c5de2c034cc5dccc8f023dcf4'))

CURRENT_HANDOVER_REPLAY_INPUTS = (('src/ui/smoke/reopen/editor_handover.rs', 6923, '3cf7a6698ad3787864282ae918ce27e6b3c13e7390bd531a24d4106f5d6542e2'), ('src/ui/tests/editor_handover_replay.rs', 5528, '51eebc8f0922d7712edccf1fab5896d0d8cb57222dac327020c6869fa23b0a66'))

CURRENT_EDITOR_HANDOVER_INPUTS = (('src/editor_handover.rs', 12878, '8b422eff87b23b9696765e82cca3dda1407823385ec2887cd9aef7d3b01b6e4b'), ('src/editor_handover_source.rs', 3316, '858dee9fa86c4fa1c6653c3813e094f8c5f3a8fbe818000fc5723ce67eae76ac'), ('src/ui/editor_handover_page.rs', 25989, '26f8465bc354dea296377bd8617932a82237113714e61a3b84e3ee3cbbf09a5f'), ('src/ui/tests/editor_handover.rs', 9606, 'f4c84cb5b7ce93b68336d65ed0c271c4153fdf483ad7b726d92fe8b8cca3d98d'), ('scenarios/authoring/editor-handover-v1.json', 36856, '5cf60907bc79ceee254b1398839aebdf34bd7acc419ab62093a206fcfd2219e3'), ('scenarios/editor-handover-lessons-v1.json', 41521, '3685bf648d2a4ddfd97583af3121b2d3650e7fbd82c7a346792d1e0d2848ebf4'), ('scenarios/editor-handover-units-v1.json', 435, '16c5a3e0d4520288c0fcf7d2aa937657cdc4dee6f7f741376b1a74d9512a1ce4'), ('scenarios/boot-entry-lessons-v39.json', 1538677, '3637cf48e03ac2dd48706f6f76c388501852c39c9cb864e5e67f929498e99738'))

CURRENT_MODAL_EDITING_INPUTS = (('src/modal_text.rs', 8188, 'afe3ddd6f874feea402e8886e69dccb287b451c8cbd377144868b3e46949407e'), ('src/modal_editing.rs', 21126, '86b2594d800b0596874fd7cedc83fbfa4c0447cd99bb87615a5995c90175a702'), ('src/modal_workspace.rs', 22739, 'ad45ddc9029601bf46cddc2117231e4472aeb7e2da0639e6f37c4a32f7a46800'), ('src/modal_source.rs', 4322, 'b5fafbf8c734be4ca7035fd111fdd947b5413608d2c3c15dd1649cd4ef0a455e'), ('src/ui/modal_editor.rs', 10354, 'ebc841c1fb08436c8c8837221d036a1f87e0e04c6ed2bdca893d4c5a0c95e2d4'), ('src/ui/modal_page.rs', 12909, '164802583ce3e55ec31e8aa89264e03754c42d7f0db02e26ee10557b6e592f2f'), ('src/ui/smoke/modal_lessons.rs', 28569, '73163773bffaa9bcd1e36e2adcd321e20f3413a749b66a3e25289fd2ecedc9dd'), ('src/ui/tests/modal_editor.rs', 11005, '658425aed76a54d635890437daeaed60e6cacf149fd1563cf82490fb075b9082'), ('src/ui/tests/modal_lessons.rs', 7644, '2b65dde47096d3a1cdb655722284803aa133bcba0131893a5d57e59bf4b90fdd'), ('scenarios/authoring/modal-editing-v1.json', 44129, '46d8ee42767e3d657e4ee2c5c0c6076a3b5dfb31578bba3c9e7c5de05bcafb56'), ('scenarios/modal-editing-lessons-v1.json', 42651, 'a30ed186850e7aa791b190f58092c59cb55c632276886b25cc134f85ceb3acfc'), ('scenarios/modal-editing-units-v1.json', 519, 'a125b98a43b705e15212ca4fcbd0181053f2997d744aba4319d8e687442b4450'), ('scenarios/boot-entry-lessons-v38.json', 1497203, '56fef968b67365f03a0d41b140f65ebe81a34725593f19047490add088ad3864'))

CURRENT_SSH_ACCESS_INPUTS = (('src/ssh_identity.rs', 8519, 'f7bdfb14fa932facee2ef110ebfd733499bb5d039219e2e65a36d221326d46ed'), ('src/ssh_policy.rs', 11771, '5d4cc432ddf486d8d5b6970ada5bca8889c9cbfe8724903b79de0e83bc844c53'), ('src/ssh_key_access.rs', 4612, '20fcb7a2d48827d6d580d87380fe55fbe6c2b319d4380d995669bc63dc0e266c'), ('src/ssh_access.rs', 11169, '125e01a50248a96a9d222dfa630329f2201cea410943aab64a8154f60654f047'), ('src/ssh_workspace.rs', 30797, '6d62ccc180463e493b8377c319056b98a57458519588b86aa4871c35d30321ea'), ('src/ui/ssh_page.rs', 18929, '417bb0a4a1efbd6e9fa67b1bac5fbd498779b088ef58ab2e89a664476903ffc2'), ('src/ui/smoke/ssh_access.rs', 16638, '49e0c2d2cbe2d6ac8ab10eee25ebee1103ba720314cd70f9e755d81476d71cf8'), ('src/ui/tests/ssh_access.rs', 8982, '3c19afbf700b88feab03f4a8fba2471e27bfecdf8ad49f2f89e299dd837feb3d'), ('scenarios/ssh-access-lessons-v1.json', 108312, '308b4012f764c918688b1790d902fc3ee95f671bb902fbaa1ea2b59b2aa56393'), ('scenarios/ssh-access-units-v1.json', 897, 'b28a735225924a23ee46cfc32eb6f94d9f02e0bf17e7176a1494ea53018fec3d'), ('scenarios/boot-entry-lessons-v37.json', 1454125, 'd27023e6eec74666814283f30c17f9dbce0ade11704fcf9c7c42c7b9b806bc1a'))

CURRENT_DNS_QUERY_TOOLS_INPUTS = (('src/dns_query_tools.rs', 10265, '04a8c278121ae426ea58d538740e467654d3ac5dcdf48674184eea2b086929ce'), ('src/ui/dns_workspace/lookup_controls.rs', 3350, 'a74982c50c7ac1aaff3e74521817055ce1c79f9b299cd7a0fc1e7f267e85d1d7'), ('src/ui/tests/dns_query_tools.rs', 10328, '6811b34dd4f36196083ab2d9d7cb57f770bdc7eb52d088e5400233eda7ddfc60'), ('src/ui/smoke/dns_reverse/query_tools.rs', 5890, '687daf2f84b7ff750d4e4d2744c7895016eec544f0b63a554411e961a681b157'), ('scenarios/dns-forward-v2.json', 141199, '72c36b7487c733f871ad24a3b8e67d49a75ed0789c84f08453714fee7290bb70'), ('scenarios/presentation/lpic2_dns_queries.json', 24256, 'e3bbf5117b2bda4ec68fdde5d0c9b57ae5b99074473bc4248221cc3f5664409f'))
CURRENT_CLIENT_DIAGNOSTIC_INPUTS = (('src/client_diagnostic.rs', 7539, 'd3551f4f62d2bc77bc64b6d441f5c93e8630c7233f1f0bb9a067df15c898cef4'), ('src/ui/tests/client_diagnostic.rs', 8735, 'f6a9b744c7d2717387665134cc8bb0d3c2df79cab394818426832152f8cd5710'), ('scenarios/boot-entry-lessons-v36.json', 1346286, '15b6bc47c6bfcde9aae9e2258d6d7b3a1f02e81b5e0828f2166766ca2d73bbb7'), ('scenarios/client-diagnostic-units-v1.json', 478, '66dc3f62197b39f0ad08bfb30b6ec4967bc77b081f14d2b8c2e7ec489466ba7b'))

CURRENT_REST_CONTROLLER_INPUTS = (('src/rest_controller.rs', 24010, '5c76651cde537a6100d41f4366484fd68489c831d5aee6bc968645be1e8844c1'), ('src/rest_workspace.rs', 13229, '642fbd00321fa441e197d50cd9019fc3e21906a12b5e28e94f4df25ce4e9bd75'), ('src/ui/rest_controller_page.rs', 18364, 'fc626d2b58a68f8ff5cdec029e99294e518628afc1b4dee32a7a9ba99729c77d'), ('src/ui/tests/rest_controller.rs', 15709, '7fc2619dd93aa2fc5d62a1cca34b4ed0f3dd40d371ca829a043c1b7f3ee69c02'), ('src/ui/smoke/rest_controller.rs', 22754, '1d32e4bdc5fcd609e186e8b2fd30b9897298cf0f6e902a669b5e5d5428774aeb'), ('scenarios/boot-entry-lessons-v35.json', 1329404, 'f64bab18a3450b16cd920febe1d78aac600c90eeca95bd96a759a36f532b581f'), ('scenarios/rest-controller-units-v1.json', 460, 'c28a42f9f089e123c8edbcb6da0d703d7daaa107f6ef009d80670cc942249dcd'))

CURRENT_HOST_ECHO_INPUTS = (('src/host_echo.rs', 9738, '0b478277f8999691090823193b7518fccc7f57e3787737b9e2cbe67f4199e3c5'), ('src/ui/smoke/echo_diagnosis.rs', 22209, 'eb957c15aafe88a4940b92a5762add749c18dc7ed777863a4c06af9dc5af8486'), ('src/ui/tests/echo_diagnosis.rs', 11086, '64a44a227ddabd7f4df3b85842f79ccaa804bf222d0ba5d22bb0a10ffe5e77b7'), ('scenarios/boot-entry-lessons-v34.json', 1284622, '0bea79cc52fd912c832792c25245b5c3c6a4bf95bb92a70c3777d984c7b78c54'), ('scenarios/echo-diagnosis-units-v1.json', 484, 'f14a0406b586385ca0fe8d4a726ea1d80b2577ab4b5b880a3d75a6a1e91b2882'))

CURRENT_PROTOCOL_EVIDENCE_INPUTS = (('src/transport_core/echo.rs', 19233, '89cc0f0ced3b857101b3ba7bb518559b0234888327b3bc3ce258440117a16f3b'), ('src/transport_workspace/echo.rs', 3637, 'a7f2abaeac257fc06f51b406e9291828faf25fa9269e3f7f39adb388fabb328e'), ('src/ui/transport_page/echo.rs', 4395, 'c28b0d6104a2d39dd9f657b1440621c16daef99393127a61743eeadb413cf88c'), ('src/ui/smoke/protocol_evidence.rs', 27656, '27f7d7b51c69c9ef50b33e9c93ca7bf919327c3e5b87a4d16fbbd9309ee0f185'), ('scenarios/boot-entry-lessons-v33.json', 1256589, 'c050c9a708163e5d630e7600817170c9a8352ecca3abce390b5e3a9c841e41f9'), ('scenarios/protocol-evidence-units-v1.json', 477, '05324b1ee58dc8511102ba1ce34b6e661c2c2fd06d6904cec304954550023db7'))

CURRENT_MIXED_IP_INPUTS = (('src/ui/tests/mixed_ip.rs', 9140, '5d5fbb496de12fe7c310975494b864d2ede0d2aa5fdd3c08cc6c3b87f0df8471'), ('src/ui/smoke/mixed_ip.rs', 22541, '813a0fc26964954446205316cb423cb2e450b57cabbdd25cdeba3e073014c471'), ('scenarios/boot-entry-lessons-v32.json', 1214358, 'a0a85914d06dede4f98bc0e08a585ecd20ba392c66b75082463d41cd6dcdabf4'), ('scenarios/mixed-ip-units-v1.json', 429, '77ee7557f3a83a73601710d9d3649b13e2f2e5b8c35b8d9a833095729f4c4f8c'), ('scenarios/presentation/lpic1_foundations.json', 3174, 'e03e191b65cf696ae158258b457f251a4d0b4851967a571b8d438ee35ab75237'))

CURRENT_SERVICE_NAMES_INPUTS = (('src/service_names.rs', 12995, '0ab41960720f6365c1422ce3e02ad8739e26d074586d5c2e3b50208ff84840d3'), ('src/service_names/tests.rs', 6795, '1fd512a32f14d77b0feab2bced300193049a614b93543b552391a0476dea7a7d'), ('src/service_clients.rs', 7462, 'fb16ff2caad48a7d34b2bef26bebd1ac48c42d98cade4f25ed9acd1b59b3c740'), ('src/service_clients/tests.rs', 7196, '4543dc8a649e34059bbd202b8eb390c082126567d4cb8d9f918feb48d6577ca8'), ('src/service_workspace.rs', 21962, '4d3fc4a9f11033450c2823cd1bbf7b96b7fc33dfccca8b9bd9bd81e918122303'), ('src/service_workspace/tests.rs', 11764, '4a75b42b041ebc02e01743fb1b299b3ecf1415dcb2453b98cc4c34aadc691fab'), ('src/service_workspace/bindings.rs', 3744, '8ef128800dc3c578e06be0a40ef22b6c9a59471d054e33a8b5bbbce49210b3b7'), ('src/ui/service_page.rs', 18590, 'f1ef02a37b7d0738e8ffefcd279944953bc9ed1d5d780d56ae92b4828666be50'), ('src/ui/tests/services.rs', 10643, '7bb5da151bb1668cf52d947976ebc62dc98dc12131a1a8ff83230680909a3e68'), ('src/ui/smoke/service_clients.rs', 29873, 'dfb1a505cda6742fc9b98d18b3b5f4028da8b860177b0385ca365e57b1269572'), ('scenarios/service-name-lessons-v1.json', 37549, '1dd97283392ae63b9c69d45e25a34b8bc2c65c275963f04859d8d398a91d00dd'), ('scenarios/service-name-units-v1.json', 498, '3b3ea03ff05e39d4c6977b07c9db6c27551bb5915f83fb36161a958c3dadb778'), ('scenarios/boot-entry-lessons-v31.json', 1179718, 'afe145625c23bda350f5fde5e7653d67f0bab618e26b099d4dacf10f10eb1132'))

CURRENT_REVERSE_PROXY_INPUTS = (('src/reverse_proxy.rs', 26322, 'f3b6c81c061c93c935bbe210df382cb4c5ea5c288a6264c109468e9edcc1af67'), ('src/reverse_proxy/tests.rs', 13356, '265f5552d467edafabc3222a8a83e3ec4684c35b995b9f68a4ddc5b5beaca716'), ('src/reverse_proxy_workspace.rs', 18127, '163438e9ef72b75c7c9200a8172b3b0d9508159257559fcd6bc8ced29def355b'), ('src/reverse_proxy_workspace/tests.rs', 15277, '330cfc0c1abb3f5bd204f4b13c2a66e4d357ad230672acc7c429827f40d0b2af'), ('src/reverse_proxy_workspace/bindings.rs', 4133, 'ae44b91990f6b4ce415831731e28b2ae86d6ebb529dbc23c444fcfd7ae7e408d'), ('src/ui/reverse_proxy_page.rs', 19628, 'b13af4660e748a8e13d18b949e943d4d8871b3c46aabe400e957838a058bd456'), ('src/ui/tests/reverse_proxy.rs', 16359, '1a62fae43c800784a3fde181128212d388bb629f3090b1c20ce164f8ee39b7af'), ('src/ui/smoke/reverse_proxy.rs', 30124, '86c9505e3094aa7fed9b9763f71bc5911e0ea756c5512e7d69035b053689538d'), ('scenarios/boot-entry-lessons-v30.json', 1142216, '2ee91f59db551cb35c6b6cf64d3960ac59ff0bd43f156809fc288d5a8ea63eb8'), ('scenarios/nginx-routing-units-v1.json', 525, '8eb190961296721bcf6892c436025ef973dff4e029a16eac9fca16ed4946d043'))

CURRENT_HTTP_CACHE_INPUTS = (('src/http_cache.rs', 14108, '316010dd673411e14cc47e721c021b0b3fd9a002700acd91588e5c813c2d86c1'), ('src/http_cache/tests.rs', 11418, '3c871a672285b9fdda64a269c6a826eec71f3e88ca4308ee70e9b47a672642ee'), ('src/http_cache_workspace.rs', 19218, '2aecf66faa011deb141d3e5cd8ee71907fd1c877b5cc1ce901558b7f37915b59'), ('src/http_cache_workspace/tests.rs', 17407, '43691275be41b37043e036c12cb1fe194ef7e0bcadcbda16c27a9312462645bc'), ('src/http_cache_workspace/bindings.rs', 3736, '8384ea52f794b643c8ffa6e64dd872af012f2bbe8db2a60a315e9ef439804eed'), ('src/ui/http_cache_page.rs', 21794, 'c0f64027e417c79f0631631aa095c22a0e0230822b1e6d906fe677bf5389cfc8'), ('src/ui/tests/http_cache.rs', 11702, 'effdf08efd3d2edc887ae587cb596dc969d13288ed49a69c7a65579ff76a54ca'), ('src/ui/smoke/http_cache.rs', 28100, 'afd65f2093db95b884e10aa6c7f397cbe5c652c2989d3dcd7175a84f60b59cb0'), ('scenarios/boot-entry-lessons-v29.json', 1107230, '3c6a94c3e9c29c72f5e030281219a67361d1a9de30057820eb2df69157151834'), ('scenarios/http-cache-units-v1.json', 481, 'd3483907404eae1ca44394d5bd195d4762689b583f519efcd5804a95d81698d7'))

CURRENT_PROXY_POLICY_INPUTS = (('src/proxy_policy.rs', 23762, '6fa4cc14c17bdb9921df1caefa556024508773e137ddb02ef31c257d207d9af1'), ('src/proxy_policy/tests.rs', 13566, 'dddcd01f66885809a27f332344dcb06237549e2d784af90b5c7978d5c41ac4a6'), ('src/proxy_workspace.rs', 13871, '6bbcf5a75a589071131d9f08c114903b5bbf4906304693f016b6be1c31bc4cbd'), ('src/proxy_workspace/tests.rs', 14049, 'd0f117ceea515b7cc4c7ab54eae01e5f6a69559d059c042fc2127ed4725f26d4'), ('src/proxy_workspace/bindings.rs', 3970, '0094fe8ca7719bf8b0aed1da2510b5d26b058a809617349e9e589eec6da0e7aa'), ('src/ui/proxy_page.rs', 20131, '1a6678107edf63ec086768c71492f6c42034e4003a60d1aae2d1841f3f114859'), ('src/ui/tests/proxy_policy.rs', 15217, '6c0cfbdab5a9abf898c16171b42bb1c360d9b8642b0dbd6fea56f947289ee156'), ('src/ui/smoke/proxy_policy.rs', 29768, '37be8a37c31a2e736c7683f3f62d5ad8568c106a3ce3b54737d0ae68c69fc284'), ('scenarios/boot-entry-lessons-v28.json', 1032780, '94e2a354af30d0a3e4dd9f0160da12f23e30cab42b7f36be44665dc4df28eb6a'), ('scenarios/proxy-policy-units-v1.json', 484, 'f1b7c5448315b1885fb9af43c46422615abd338681ecfc8df00eb62b6162729c'))

CURRENT_SNMP_INPUTS = (('src/snmp.rs', 10382, 'f420b4b95804f79d4f2477e0e972ab9ba966e66fa65f5885d96e67bc3300f6bf'), ('src/snmp/tests.rs', 8337, 'eb8b562d220c5ded54f70452f260eea3109366b2fe4e61ff011c7af8692134bd'), ('src/snmp/monitor.rs', 14196, '6aae6406ff1a41f1fd638d4f3407175e44a055ddb823654aa757adf6cd2027bf'), ('src/snmp/monitor/tests.rs', 8775, 'c67557da124ed9e6614bb704d48c916d5323c55c72cf2a69ec314642acaf198d'), ('src/snmp_workspace.rs', 13376, 'ad42f11d829e153098fc5513550ca53beb70795871d962b7da5df4ec791b0375'), ('src/snmp_workspace/tests.rs', 20223, '785782c7256e110516a7ce0c934d7d8cfb1d61f2e13208332e0bf66b6e052973'), ('src/snmp_workspace/bindings.rs', 5171, '26ce5d7e9a5d3287defd4bc33722d75f7654ad914d626d78b2f3ba84e6ef6990'), ('src/ui/snmp_page.rs', 23353, '9b0736b879b6c0dae2381e275d27e0f6c22d98af0dd97db1be07583261ef2248'), ('src/ui/tests/snmp.rs', 12440, '1df5d64152a3219679dddbb63ceeaa905d816e4871cd598405ad7549a1cd8672'), ('src/ui/smoke/snmp_monitor.rs', 29533, '07eeb0de89f2c64936cdfaae50b5bba00c2a895136a37f44ed4d285bdaa3b573'), ('scenarios/boot-entry-lessons-v27.json', 999697, 'f5450836a9e070ee6f1ddd282c099cce71a03b644edb6bade2d667d52ed34236'), ('scenarios/snmp-monitor-units-v1.json', 511, 'a2f976f1a73df71adc4b0cff3699f3b28a2d5e4527435c4c57d5d2b47e6625c5'))

CURRENT_WLAN_PATH_INPUTS = (('src/wlan_paths.rs', 8078, 'eb8763db99b7dd9c513a30ee5712a9fdf5086d179b5897bf84539fbcdfdc8486'), ('src/wlan_paths/tests.rs', 6522, '639add76f21e704a41b18a056a1fa3e3890fdbb3557593c6e0187bd1bc00a6c1'), ('src/wlan_paths/fixture.rs', 1395, 'cc7afc0b3a0427f8da2a94778950c1fd4379bdd94b93e33a19517805ee3e9ea6'), ('src/wlan_paths_workspace.rs', 22146, '0586eec0c2332fcf57eb3771cfab079519fbb52969aa2ab5156e715eb5338073'), ('src/wlan_paths_workspace/tests.rs', 23050, 'c44a1c99db882bbd5da4d6765bdeda680f84461544e69edb45278a026eec1ffd'), ('src/wlan_paths_workspace/bindings.rs', 6626, '8d5b96ff9f1d14d81bd4f447170cc07959d2124295d3b2e313b904c0698a78c0'), ('scenarios/boot-entry-lessons-v26.json', 947795, '3cdf43401f2ab6d2b8e5d7f7cf83b3a6b8f99903995b5736ac5bd13276e2cee6'), ('scenarios/wlan-path-units-v1.json', 520, '4819c7baca219d61c93878c356107027e4790cd0ae12db01aecadf1de1b0de2e'), ('src/ui/wlan_paths_page.rs', 28576, 'a7dabf586b44398269d90e2471edbcf48395d8473737239721d0c3d66444cd9c'), ('src/ui/tests/wlan_paths.rs', 9796, '49be00667b3fa400862c7c115f04170159879c940f73abbe563eaaf2dd1cd109'), ('src/ui/smoke/wlan_paths.rs', 30961, 'fdad6d9afbcbf379ae71813e0125621044654b698e31457c27f62d622cd9a934'))

CURRENT_WLAN_IDENTITY_INPUTS = (('src/wlan_identity.rs', 11683, '68a5cd2e92bef6e8cfa972711afb77660482287a4b1fe6eb99c9a63a98241377'), ('src/wlan_identity/tests.rs', 11087, '805ef588acf88fc890c58282e8a9a03de189e949d8b3267ad5c32a7eed00d542'), ('src/wlan_workspace.rs', 18729, '7df292f8cf20eef25d1b6648e337eb5375b8b44bc869035b8dd183e710595ccb'), ('src/wlan_workspace/tests.rs', 20470, '8ecf599e4a2df98e2573437bbce67e31eadadf962aec8f13267aacaa9d0b3ae0'), ('src/wlan_workspace/bindings.rs', 4492, 'f9e1bf187662ed7356bee2b1ef553a4c2d6be32913c723ed1d5d07f27b2091db'), ('scenarios/boot-entry-lessons-v25.json', 900318, '60c1e12c1352805b9b422f6b697c8bd10dea59ae9910a5faf796a9f5d0dcc170'), ('scenarios/wlan-profile-units-v1.json', 480, '9454574ecf048c75e7474b54b612c140f6fd3e704d2168931fa43dd2ca427884'), ('src/ui/wlan_page.rs', 25820, '9654f2221ad0282c5c677e38a16466562859135cbe08ee1f4971a9c15f279f86'), ('src/ui/tests/wlan_profiles.rs', 15014, 'f685702f5a9afab1d04b57cabf3a3d64fed5fcc77c88811ec7c8ae668ff22e5c'), ('src/ui/smoke/wlan_profiles.rs', 31283, '9948f8149df22bcc848c980216a0cf11f4db261bff14df8017a824c7882775d6'))

CURRENT_WIRELESS_CHANNEL_INPUTS = (('src/wireless_channels.rs', 8695, '0d0a5bc6669f3eac7c26036c6eba21d7c2c9d6ad33aa534f83e98726bf02642b'), ('src/wireless_channels/tests.rs', 6737, '25e4a82eaafc67863ef5b3a997e275d2c69dcca5776a55a4c79da4957d2a2844'), ('src/wireless_channel_workspace.rs', 9911, 'ba4266478d3fc9715c3b8fba85be551fd0d2613a35aeaf363c13792879029423'), ('src/wireless_channel_workspace/tests.rs', 13324, 'fe505698700d001fe76bcbd0b47ced24005dac2ef1f2189d0595733f1243f6e7'), ('src/wireless_channel_workspace/bindings.rs', 4157, 'f0e51484a4fdd10217f79349976573e48ef15b4d224ccb2769e9c84707720a95'), ('src/ui/wireless_channel_page.rs', 17410, 'f7d79662161ab929b5bb80adf3f41867faea2c7501a6c8f3dc5f04bf95c8ea30'), ('scenarios/boot-entry-lessons-v24.json', 860182, '3ca83d0066bf7f9614c84667b339532bbd9f41af98fdd3843f0c839ff92344fd'), ('scenarios/wireless-channel-units-v1.json', 498, '7670ecb8a1ccc9bbdd5c2452b4d73a02225d8e540f7336f5cb5e786ab8798f93'), ('src/ui/tests/wireless_channels.rs', 9639, '85157ee043e55eae0fdc39804da1d006ea6e36120806e1cbe796c4b88f79227e'), ('src/ui/smoke/wireless_channels.rs', 28986, '854e5b116e92a25e6c8c7643699604f6cb83384ec9b4dfe55e3d64b7fc63e646'))

CURRENT_TIMEZONES_INPUTS = (('src/timezones.rs', 6582, '50e66daa74fad6567273c0a3c37bfd04498b00e471af4570bb1c25c9b6292aed'), ('src/timezones/format.rs', 2696, '3f233f24f04fa0276f131af58af6f5e27eb46c83704c8149b5f9c6e9af644e5e'), ('src/timezones/command.rs', 8606, 'ef622787a4c07b02b6eb92ac20237b155ebea73e72e891ed99d25d1fbce265dc'), ('src/timezones/command/literal.rs', 3748, '34aa2426620e3ced2d5bf757ff4d06a55b07a63a6c80c8380e82113e1c2c3d48'), ('src/timezones/tests.rs', 8895, 'b5bbf0d208cb2513e90849e9b48a05a12c52f520bd02d80dd4a3532264777b01'), ('src/timezone_workspace.rs', 12314, '3b4208f72cf763d6d382b8041748e9bb58fb43a350e9717097ae2e0f1a1a3d59'), ('src/timezone_workspace/session.rs', 6750, '9416396d4d3773ac62589a420bcda2bafff92d4bff45b781ec2a983ba95c6284'), ('src/timezone_workspace/tests.rs', 19678, '2ecf1a02c25ce5fb563dfbe08add1e623d809083139e03b36b469704c38cbd1b'), ('src/timezone_workspace/bindings.rs', 4216, '51acc9ffa83ecc2755b0d60ea939eea6cb465a9d0229039bbd3e97fffac21d2b'), ('src/ui/timezone_page.rs', 21185, '9fbeb4b97b1085c08676444deb435e32fad05a4918ada831c1a6ffcefbcb561e'), ('scenarios/boot-entry-lessons-v23.json', 845613, '98a8ba114fb377730c017987050a26676db2b95115faae5f7b81612f8bae7b44'), ('scenarios/timezone-units-v1.json', 490, 'ccbb5ca91e0aa730056484a47da893d48bab97a42e99998b9c17a67ced8d0ff2'), ('src/ui/tests/timezones.rs', 9410, '1c9aeda1385b32a8565246791f745d8ab5af2ab3f9dd6176bee231d8ca75d4d1'), ('src/ui/smoke/timezones.rs', 28398, 'a6dd91b8906f43cd3b8edf8d46a4b5adc20f529d47c5d222d2c3ed6ea1ce3f2c'))

CURRENT_LOCALIZATION_INPUTS = (('src/localization.rs', 8914, '415302eaf76e25cb315deacbecc6aca9d7787a42f41fd64f87f8a80ecf98598e'), ('src/localization/encoding.rs', 3972, '32ac6677ed16eb0d0a829e54bf04372582a4a026e107fa9587ad68ce5d435abb'), ('src/localization/tests.rs', 9759, '4142fe6d5b07250afa289fee26380cc9bbaef67d493d3a4c43ee33e0e76917e8'), ('src/localization/virtual_files.rs', 9430, 'bb716e6b0095bdf0b3357dada7d26d2fc69e2df1b98f13339b7857ca4a5b694b'), ('src/localization/command.rs', 19029, '178b6e98d0b19dc4f20ba26545e3e5389502346733d52417725da8afe46669c8'), ('src/localization/command/literal.rs', 3766, '86887bfb2adad6dd63aa13578f1e36c46465193a5036b3d4edfbc4eef2f5f841'), ('src/localization/sample_report.rs', 6090, 'cbfcd8072606f6a34c0eabdf58cf83919a94cbd57c609a9f62844f9a55954c2b'), ('src/localization_workspace.rs', 13138, 'f596e9973cccac7e9de1a7cbdf68bc71993545402019813afab59671da70548a'), ('src/localization_workspace/tests.rs', 25299, 'bd59144160bb1caffde1e5d2292f8f1ad3a940c5680ac290e41a7445fbba68a9'), ('src/localization_workspace/session.rs', 10081, '271c13f398878da3de4669de4c949d8a86afad0105ee382aef5e7f085ce0236d'), ('src/localization_workspace/bindings.rs', 4452, '2ce58f3e29bc90e5efaedfd8e4ddaecc73a1813025e17b4d600bda84dc40a281'), ('scenarios/boot-entry-lessons-v22.json', 826243, '5846f733309f788fe5d63589aa2ee4aa96405522b03b9fe795183542c80d98aa'), ('scenarios/localization-units-v1.json', 538, 'f58373a9d2bddf711974099ebadec4e29231d6641a5e4a33f35f3fdf591d37f2'), ('src/ui/localization_page.rs', 24016, 'b2e290c24f60d17c55144ad62c662301fb12883d73bfc4e8a7d065547b28b87c'), ('src/ui/tests/localization.rs', 9581, '23b2387d9afa1459671e92bce877677ca822b4c08c70b6936e281b7a2b94300c'), ('src/ui/smoke/localization.rs', 36007, '7bba1eafef82dd5df0363f8f1c262820855d5883b27e1b50ee00f41b3ab89aea'))

CURRENT_PRINT_QUEUES_INPUTS = (('src/print_queues.rs', 13005, 'fc90498f37d48c8e012a62e083d215ea7320ecdc80fb26ea68bac0d7627bb983'), ('src/print_queues/command.rs', 12703, '9b32836bc5ad415f28623d71af17341fd868594d63f4e7b12c1d976efef8559d'), ('src/print_queues/tests.rs', 19339, '4b2916c489bf5ed75b53932c7aefbcb563595e837bfee29a47cda53a252b0237'), ('src/print_queues/literal.rs', 3904, '4cddf5c9d608a80fff5a4e66d9e625be51eb78b6aa35ec25e1df047c2ec428a9'), ('src/print_queues/bsd.rs', 5745, '31516e844a337787a801115848341c3da0965cc50e42b6cc87df6bbae789fe3c'), ('src/print_queue_workspace.rs', 16126, '851fb373d620aaa1c5f6ea662d6603adffda67c0b64d22cbcbb42d8a8e6b7e23'), ('src/print_queue_workspace/session.rs', 8790, 'a484bbcfb170af4760f9dc695213c08ccf9092916fdd187c42fcf8895ccd3ba0'), ('src/print_queue_workspace/tests.rs', 24439, 'e8c116ace3d55549d46f41e395834cbc12adbc163dfaa1df1ef0c9979402a6c6'), ('src/print_queue_workspace/bindings.rs', 1776, '8256c14b21a9d02b645f9fdf68aea9f68021c4c8ba81d9a738d84aad4935910c'), ('scenarios/boot-entry-lessons-v21.json', 790382, '156c6c3ff325c56f58c0f5364f8ac5d9be175e762209504615506d0c6990f96d'), ('scenarios/print-queue-units-v1.json', 584, 'aaa7e68f9eb5849a3b4116ef4b748e63065115bfd5ed58784a4caaa56fa45a79'), ('src/ui/print_queue_page.rs', 16622, 'fc0ea701a4b764e24f44c83e8132a62b511e2b0ab57851e7162fd92d93cf8b91'), ('src/ui/tests/print_queues.rs', 7774, 'c4621a43788352b02f9b7baff3b4e1b4d5514b46a08f5eb1ed05e2fd669223c0'), ('src/ui/smoke/print_queues.rs', 29154, '2a0ea916084e54314ce7279392c558ecd87de04522fa6e5c037d13ae40e98584'))

CURRENT_DNS_ACCESS_INPUTS = (('src/dns_access.rs', 284, '09ece3a5a4bdd06685ce981f17dff18517fc65c331cc6fb2929cb18522f0e33b'), ('src/dns_access/acl.rs', 5813, 'd3ca40588f5397bdecea5c869c0c3237ccc29593d5290491ab010778ca9278cf'), ('src/dns_access/config.rs', 12771, '236aa21226fd5fbbec3edfd9cdcbe8aa8c06353f33a3e825523d35cb406c591b'), ('src/dns_access/world.rs', 14445, '6814553ac2a22de961573f7c81d01175c4d9972a1078897b8a8dd53f033965b5'), ('src/dns_access_workspace.rs', 22520, 'c61d642a62d5b0549ebb335e76af9ed361f0c812988ad01d1ff334e4e1e109a6'), ('src/dns_access_workspace/session.rs', 9281, 'eae3fe9e49898850c93c577585253de0276a9fc2eb0e20eb23701dc67c3bf5b4'), ('src/ui/dns_access_page.rs', 20309, 'd46da7bcf0466e76183933665d6f1c49d5429c56acf392f50540a8a481360204'), ('src/ui/tests/dns_access.rs', 11814, '65c4e3b1943c44c2019b34899f330e1a1023e2e86a6ecfa1c9ddc0fd1fcbfce2'), ('src/ui/smoke/dns_access.rs', 30032, '3b7a16edb7aa80b21551bc01d8510aa26ce076a0ffefc9564322372178d81c6a'), ('scenarios/boot-entry-lessons-v20.json', 755433, 'b7a819602c5d952d8f145757d38954f0bad507d73141734bfc1a5e6b44722af4'), ('scenarios/dns-access-units-v1.json', 567, 'bcc6442bc7c2f2b88d5bc20a451253bedf712b3bad3605430685a344a5815ce2'))

CURRENT_SOURCE_INSTALL_INPUTS = (('src/source_install.rs', 24032, 'c2606ed38034d7480f794aa891f87e627a8da59f749845e6b4804cb7076512e1'), ('src/source_install/command.rs', 16648, 'dde57d0169cd40e636a809312c932e1426f6a278f9b28fc273e52dcd5c43a421'), ('src/source_install_workspace.rs', 25634, '1ea51c5437aebb7444b72b3105d34cb101f2fe6fdaf16fc0569d2f14f52320a7'), ('src/ui/source_install_page.rs', 18326, '9d0777b10ee455535a62cc9e061140e73e73f98848dc49be435ebcb2b8a0d9ed'), ('src/ui/smoke/source_install.rs', 31158, '2e977eb6cc1be97f531cd6487144333af1d27e2bbb83328274d19ed97595b38c'), ('src/ui/tests/source_install.rs', 10106, '258ca72d0400663a12207ea31837e55f79ab9b3b30700bdba3a007cae210a701'), ('scenarios/boot-entry-lessons-v19.json', 645693, '96bf9cbcfa9a458c1186f59564a84ce6f5a2c9febcb070d855901b01e6cf3959'), ('scenarios/source-installation-units-v1.json', 626, '501e1d69291a947a44237434eabd2f51a56e732549d9753f2e827c9fc02b2eaa'))

CURRENT_MAINTENANCE_NOTICE_INPUTS = (('src/maintenance_notice.rs', 10723, 'ef1cd372ffa036446ae634ee0d4ede747639a2318c40768d0498b5eace3917e3'), ('src/maintenance_notice/bulletin.rs', 6217, 'd95de2f14201f56e99d191341c3ba1ebc0487e5df8560cf46c078c96566058c5'), ('src/maintenance_notice/command.rs', 8199, 'a2088fe213047ed41132556904a92d2ff01af69a0bbddf0514717ad6af4acb2e'), ('src/maintenance_notice_workspace.rs', 23148, 'f26cb4dedadc73a89ea5e449197a59cc9e6bd589c0ae05a15bfd1ad4bc1f0f74'), ('src/ui/maintenance_notice_page.rs', 22304, '25ddd1fc145fef0f0d2d7ff427979f337a686d016664c3140d5b9dcc1b67480d'), ('src/ui/smoke/maintenance_notices.rs', 26643, '211a49131d5d04663738dbadd62bc8938cd8763fc5e0783037ab72386871f47e'), ('src/ui/tests/maintenance_notices.rs', 7293, 'cb3deda0dae4d7a85bc46dcc10c4df8ad082e683839b3ec5fd82725a9a53a852'), ('scenarios/boot-entry-lessons-v18.json', 592688, 'b8840b4e81e22ebc726667d071fb20a7f370fa78dbd62c340bcd02948a6a449e'), ('scenarios/maintenance-notice-units-v1.json', 490, 'cd74a79fac772c66bd14931e8358023c3989339811038757d7ba770d87ff4c0d'))

CURRENT_CAPACITY_FORECAST_INPUTS = (('src/resource_measurements/capacity.rs', 8298, 'fdfcd02e4861f7cc59cee06dcc4c5d43e142140ebbf3c2f60e201373a77586da'), ('src/capacity_planning_workspace.rs', 20227, 'e41ea14e63173323c9ff887763aa274fb9061aa2479ac5890c5448eacb77922d'), ('src/ui/capacity_planning_page.rs', 28900, 'e89e5b45ee1642b6ab63d24e6620682edf2c5bf02a81512e70f52ca5f5bc742c'), ('src/ui/smoke/capacity_planning.rs', 28985, 'd5292d0d58ab551f151fb97b7c1939f82775cbb5bbe989f433d64356e1a148d8'), ('src/ui/tests/capacity_planning.rs', 9133, '4e0a82301d3a806faabddd7d7758adf2aad46c31c26c5a3dc3544f905a76ca07'), ('scenarios/boot-entry-lessons-v17.json', 574654, '81ccdfda8f15abe6642b8e771901620dd480112b2c63c62d7cb44514393bd142'), ('scenarios/capacity-planning-units-v1.json', 508, 'c50ed1b592f64ec76611e51d10f58c30b99b841d09d20b19e3e59f0f8a419197'))

CURRENT_RESOURCE_OBSERVATION_INPUTS = (('src/resource_observations.rs', 13833, '5a2b88c38290baf22d1e3b4f6a5ba90448a60372f1920be06c89d89f9429579c'), ('src/resource_observation_workspace.rs', 31548, '76e728adc785d115161551f47da1ceb4e1d5f7292446c17948dbd120993b16c2'), ('src/ui/resource_observation_page.rs', 21554, '8d326c979554ceef1a467c9e292ec7465a2a623b20962781461d36e3577dd4b4'), ('src/ui/smoke/resource_observations.rs', 30053, '3477cc5b0a34571cd60f2b2072abc762c69b2439165b4cc39c46f6937a379090'), ('src/ui/tests/resource_observations.rs', 9370, 'ff89aad4ad9cb272ff70c2e3aa0042b30c09e39d19ac42de6288e183711381c0'), ('scenarios/boot-entry-lessons-v16.json', 553363, '0fecf9985ce5cfdf6f503ac0eff693132486f5816a6fb23a7c3cf7b447a7fade'), ('scenarios/resource-observation-units-v1.json', 510, 'f0f3e6a87bd7218727b5ba5489cf88d087f5746444ed6b235bb2345c468f27c0'))

CURRENT_RESOURCE_MEASUREMENT_INPUTS = (('src/resource_measurements.rs', 9682, 'f190aa58d53aec8af513d6bd5f43a766969a6b8492d8a51b57b9cc25506eca9d'), ('src/ui/smoke/measurement_reports.rs', 24676, '657a1d61e1015d4a81954712c3e418f57ea636890dedf70b2b943398334e6a90'), ('src/ui/tests/measurement_reports.rs', 5223, 'c13500bed12955a6afeb040fc14420c19b960049521d470626c7c1023d7a5942'), ('scenarios/boot-entry-lessons-v15.json', 466677, '9b87f979c1b43c77717dd39d2456e514663d797b6dd0b0baa0ae9144669657d5'), ('scenarios/measurement-report-units-v1.json', 612, '4a923a2d1dd196a6c040cca91bfcce1a4b1191de21fa0457af7fc91a577681df'))

CURRENT_LAYOUT_POLICY_INPUTS = (('src/partition_fs_workspace/layout.rs', 4796, '1734f87581bd08aee4554596577b41b1f49f67a514f426c7d4cccdab69681ec0'), ('src/ui/smoke/disk_layout.rs', 24713, '936c8dbb53e02c0b1b19355d1a7e64b91a5c009588ac145abc8800636cdf4da3'), ('src/ui/tests/disk_layout.rs', 9302, '09019440b71b595ef14de27ca6467f2665aea580d5c84fbcb89e29f16d9c97d8'), ('scenarios/boot-entry-lessons-v14.json', 397682, '5dd37d1b95476aed891ddaa6d36988e193f0ac2f1a939db723e30eefe3f77834'), ('scenarios/layout-planning-units-v1.json', 451, 'afe3e5575a9464b2601518692fdc7b23ff8851093f7b6e018d4783831f04ead0'))

CURRENT_JSON_INVENTORY_INPUTS = (('src/json_inventory.rs', 8976, 'e3cadeed2758ff9468d5b1fba0e4ebf50fa805b9df375afcf14a89a106f251ab'), ('src/json_inventory_workspace.rs', 10369, '71aae30d5a1d76ef7d2abc3b675bbd48c8cfac9ae6c2b5f573355021636978e1'), ('src/ui/json_inventory_page.rs', 14087, '63df719b60c769a6af5a9d3b4939ab1772f7954513ffe0b2b65ca956652537ff'), ('src/ui/smoke/json_inventory.rs', 21081, 'f197c4251d461c896edbd73508ffba6816f619716858c1c68470ec8adfd4a275'), ('src/ui/tests/json_inventory.rs', 10892, '270997a8046e2f994ad795ae141d04d1eeb714e45dfbbe84fc06a6ec27efb84c'), ('scenarios/boot-entry-lessons-v13.json', 346659, '622bdc1e13ef59f9d2686b45ea240e8c55670c5c707a11b888fbc45ec8fce810'), ('scenarios/json-inventory-units-v1.json', 431, '6e8955c60ee90e853dc9fc23cbcfafc530cc81b01077f72b39e042619dd3a867'))

CURRENT_WIRED_DEVICE_INPUTS = (('src/ui/smoke/capture_size.rs', 4558, 'f3be02e97cd34b11537c4a184db9321a79fc86ec3bcb9ca6cec326954ccb3b22'), ('src/device_path.rs', 5583, '946143fc0e54d538d88bb89ed3642d8eefc9e260f0ec4072f2516bda551df56c'), ('src/device_path_workspace.rs', 16115, '17a421f62a9fa8647486d3784460d078f00d5d31aa4de5557d301996ce7ee734'), ('src/ui/device_path_page.rs', 18652, 'c9db16545caa0cff9825e06ee2fc41e0655ebe8b81e266d3d9e65bb35b739085'), ('src/ui/smoke/device_path.rs', 15133, '6aa9b48f859195b1a2cc140bf40645e729809710314f741f89c971344ca935ab'), ('src/ui/tests/device_path.rs', 6875, '265fc12a81d13fff5ae1562f529ae5e5a7bbe8275e6d3f32806eaaf8d8b99ecb'), ('scenarios/boot-entry-lessons-v12.json', 295677, '57bd0ea28dc8f821ce9d522e6e6d08a5eeefa117104bdd93c5b420b14624d517'), ('scenarios/device-path-units-v1.json', 430, 'a12d7087806f83122f22a1956b236b66afeca5c259824a56d8bb7c791357161f'))

CURRENT_LOCAL_MAIL_INPUTS = (('src/local_mail.rs', 7372, '436c1e29432d0f8dfb343c964f90e4df301974d7404fa5d58868e2a6d8c239f5'), ('src/local_mail_workspace.rs', 18801, '72a3c71de36d5f57a699909ba5c50dee8a7a6f6f7ec22fa05f429ee0de4fd291'), ('src/ui/local_mail_page.rs', 14504, 'b69b72647a163915fcefbedab9926c38d499cb75531a99713079a39b71423e8d'), ('src/ui/smoke/local_mail.rs', 15903, '916217fbe9f68b6f1ba73508303420b264e24e9c99f6b596863ca41a95154681'), ('scenarios/boot-entry-lessons-v11.json', 219322, 'eff9d9baddf8276e30b5fa00c1b4996092156cbae36e277502beb869a05188bb'), ('scenarios/local-mail-units-v1.json', 503, '2702560613e1688f9cb089d2adbb8cbf6c3c1aad69efc92812a3d7f1d1938858'))

CURRENT_COPIED_JOURNAL_INPUTS = (('src/journal_archive_workspace.rs', 12690, 'b0e698b7d069d684dac7c824ff42ac77b168afa29744fd21626621852fcda47c'), ('src/ui/copied_journal_page.rs', 9565, '281b2d1a5a8f85f04b0d7d0441e3e2edcb6c4266aec23b99510f124c02947232'), ('src/ui/smoke/copied_journal.rs', 11809, '8941a74ea2e91a888d1c8948966ad3e6d7442a8faf9491639371db18394e9ed5'), ('scenarios/boot-entry-lessons-v10.json', 198073, '3054b7059766eed48e6ba5ba590ec8eeb9250dda462129fbc1056d405c4615a3'), ('scenarios/copied-journal-units-v1.json', 428, 'be7c827e9e6e5d6f1a5dc22e83d1e761d550eb5e56ba69f2498f54fc3d7be930'))

CURRENT_COPIED_JOURNAL_FOUNDATION_INPUTS = (('src/journal_archive.rs', 10290, '2b59e4ba4c106a24c9b721499b168641e2afd61f097c1ff66ae0fa5ab319d027'),)

CURRENT_CHRONY_SOURCE_INPUTS = (('src/chrony.rs', 7850, '88c9605ea897485e54cdc3f732c9397fd0844eb9bb7704c1806851fc281e0f02'), ('src/chrony_workspace.rs', 13156, '6039e5b0e13635c6181070001f12c1d9b4982eefee24d07f6ef9add176181516'), ('src/ui/chrony_page.rs', 11045, 'd65fc4b57c4493dd2e79fcc1828d1634cb5b7e1ac4cd334712a8fbaa8c75807a'), ('src/ui/smoke/chrony.rs', 10752, 'fd58313e675ce6a4b4518f19258c9bd607281996ca2aa61ab64bb54bdfd2f2e1'), ('scenarios/boot-entry-lessons-v9.json', 189320, 'ef245eeabcd5d5502a039b34d8e80eaf19f504bdd7293db1769c34e76ea429a7'), ('scenarios/chrony-source-units-v1.json', 453, '9040352853268e51da39ae712e5103c9eeb9ef1dd2597237d877b03fd668f5b9'))

CURRENT_RESCUE_STARTUP_INPUTS = (('src/ui/smoke/rescue_startup.rs', 8538, '0f85a17812016f8f0fea27e720d0ee6ffc89bfeeafd7b9273716846ed8fa2cc4'), ('scenarios/boot-entry-lessons-v8.json', 182907, '2a25833145e2190665b70ade20903eb511bd59957ff9fc225f14ebda54ad8df7'), ('scenarios/rescue-startup-units-v1.json', 455, '09cab8285483fc167146f400db4ee2b5913cc84e727120c31e13fac1635ae927'))

CURRENT_GRUB_ONCE_INPUTS = (('src/grub_menu/temporary.rs', 2544, 'e8cb23594d67d23a6d4f1581920e6a2eaeda87ad6bd623d0b4503da58067eae2'), ('src/grub_once_workspace.rs', 12420, 'ce7257c73b7b33cf9d7f271fec848f79d03c6be90daee0118c30923c11265a72'), ('src/ui/grub_once_page.rs', 14222, '690870337c1f3ba7929c551431d9b5f8fb05e031cebcb8535d38e0f7ef029697'), ('src/ui/tests/grub_once.rs', 3865, '23429393163dff0759694035276951f30d636fc758201ce580ab73efc063d663'), ('scenarios/boot-entry-lessons-v7.json', 155381, 'fba60017522c8355b445aaf61ac15d6bfe6d2a45df51fd425e598e5f003aaa2b'), ('scenarios/grub-once-units-v1.json', 435, 'f5142837e3c023650870010e099264e51cd18120fa6b57a1d5ec7fb3ed43c6c2'))

CURRENT_EFI_REMOVABLE_INPUTS = (('src/efi_loader/removable.rs', 4605, '62909c0c8e888bcc165c310d24003d69936b935bcf6375e1ade426a53ede0cf5'), ('src/efi_loader_workspace/media.rs', 6298, 'dce3ea1ec47f01b5bf9935ee54ded6595f8284e9946259b77f33efa3618df285'), ('scenarios/boot-entry-lessons-v6.json', 146910, '532402299e280b04a63c6b4af13b91aa27162d0b3d30d8f22b1ae82e0a721643'), ('scenarios/removable-efi-units-v1.json', 454, '424fb367061e1a6ddfceb2d5bba25eea756a08280cd6401363d6f4c1748d609c'))

CURRENT_EFI_LOADER_INPUTS = (('src/efi_loader.rs', 18309, 'cee408d2ed87ea882436ca5410b982e2850b41b1e916090081428cd83a88a0ee'), ('src/efi_loader/commands.rs', 11877, '4bb2b0ca85caec216a765269f4858a5ca6b2fb8cbd18c3d56e0da2c67c244b8e'), ('src/partition_fs/fat.rs', 5070, '32c57a5a2f4cb35f26b1ec4c4e7f37a175a1b0ce90fa8c0f927a489743dadb34'), ('src/efi_loader_workspace.rs', 20027, '71419b93fa48358e420d6ec1c987747d8b58a3dd5bca35555efdbf7f7f8e59db'), ('src/ui/efi_loader_page.rs', 18037, '57d2fdd87506bd744157328f7f8515ba2cceee0182320006e0403b6376c88135'), ('src/ui/tests/efi_loader.rs', 8371, '6d584b72b5a55a62ca838095c5eb97968c2a2715efd87e697dac16ed91020c3f'), ('scenarios/boot-entry-lessons-v5.json', 116179, 'eb488064fe6023f9b15181e6a15e5d1ebed50665f4bc3bc46fb098e3157aee09'), ('scenarios/efi-loader-units-v1.json', 458, '5c4dce71b3de7b426892a17a234d1625b8f2c63646ad027e928d135bf00cf6dc'))

CURRENT_BIOS_LOADER_INPUTS = (('src/bios_loader.rs', 24971, '1349608bf78ad7c2f53dd21a65405679d76fade83451868f546598068a5e5dac'), ('src/bios_loader_workspace.rs', 14534, '578d10b4766ff4eb67721c07b0b17d65a4a535e8112090b14b865a9156c3aefe'), ('src/ui/bios_loader_page.rs', 12580, 'c563fa1eb004ad9fb07b68e07da5845f8379ba092de1f5e364896eee36b8e4a7'), ('src/ui/tests/bios_loader.rs', 8019, '608a5742a4dddbe089aaf79b17f13d8976c487cd905a8f224e45e71065c30342'), ('src/ui/smoke/bios_loader.rs', 23315, '65e49704b9bd1dfea7bdd43f9c6e110d2cc611f80038af817ba36cc0b99a4308'), ('scenarios/boot-entry-lessons-v4.json', 92973, '9cd1f1ac4b260aee0dcf417c99581268304d41dc9f45394dc37ffd8bbd6b7980'), ('scenarios/bios-loader-units-v1.json', 483, '190b1b0902c3acf9e071dc05157e821588cb7ad8b83f73d9bfdfd37f1b045a9e'))

CURRENT_GRUB_SETTINGS_INPUTS = (('src/grub_settings.rs', 11508, '247c58fe5b61dbed27c8e271ee4997fa0741422eb2b5713a557dd51fe5b7f4ea'), ('src/ui/tests/grub_settings.rs', 10652, 'c7088d303bb473834feb260f8b59c29fe8d406f580e85fe8258580f6e9fdba63'), ('src/ui/smoke/grub_settings.rs', 16056, 'bf3331fb53eff1a048d13aaacb5ec1a17fcb341198190dd44cce111b4a2be132'), ('scenarios/boot-entry-lessons-v3.json', 75499, '10abc5ba29722f5b7103c43147df0decdb3251f692cc51440400bb95e6340e19'), ('scenarios/grub-settings-units-v1.json', 445, 'bcb6250ac6face65df0b5712dce02bda115f786549c897eaa7a1545aac22fd8c'))

CURRENT_GRUB_MENU_INPUTS = (('src/grub_menu.rs', 20164, '8929f471a8c7c21d0688bfbed539c3f6cd496eeb2f3717a7163e9ad22b48ecb2'), ('src/ui/tests/grub_menu.rs', 9842, '1ca64620adaf0dcc784e3fc97050b2ba723553017695fc3d533db03c14a6274d'), ('src/ui/smoke/grub_menu.rs', 15081, '4b296c7741c8b95622f40299b02367efdb164d63ef9ed0302446801bf3cccc92'), ('scenarios/boot-entry-lessons-v2.json', 53128, '5db52bc84802b18562b41e2c3293c3298e07f8d521f379cf22e7332b65759086'), ('scenarios/grub-menu-units-v1.json', 431, '371f45c687854be29a96f0b2e611ec3f0113906674fdf49b96565ef725f1a3be'))

CURRENT_BOOT_ENTRY_INPUTS = (('src/boot_entry.rs', 18949, '405e08ad39141587f9f10c68204429362e5ba5893c6e80903b88c0776c5a84ba'), ('src/boot_entry_workspace.rs', 23179, '2b72e3bc49fdd6708fa23713d398d01074c88631fad3fc240a8d0a3815d15f54'), ('src/ui/boot_entry_page.rs', 20756, '78eab1a7932de66a807e3e32fa307ec5866978436d69a2d76d87102e6ef62fed'), ('src/ui/tests/boot_entry.rs', 11738, 'a1cf7ac35c5c78dd5fbe318860ab25d877ccbdd74cf360f4f2b750c967255db2'), ('src/ui/smoke/boot_entry.rs', 19799, 'cc7a25c5e822fd7b8084eeff8ac28c7081108f231662784a39d525d865368202'), ('scenarios/boot-entry-lessons-v1.json', 28020, 'ce351b5fe6a4827d0080dd00b75211fbe4547cbc5dfec0c13e41c078b780e2d4'), ('scenarios/boot-entry-units-v1.json', 533, '97fa6cb0f86d5644073698cb6d573732989029d7fe09d9428782124f3b53b407'), ('scenarios/kernel-module-curriculum-v2.json', 71683, '1537c67f086d3b60be811b8377afc2ad7a3499e61abd9c0c84ce6b128b53273c'))

CURRENT_EUI64_INPUTS = (('src/ipv6/interface_id.rs', 3471, 'bef3e3ecb9a6065ff8733ad12414ae2714b29584542abe46f54becad89201b08'), ('scenarios/ipv6-interface-id-v1.json', 15160, '35ca8a944efa9698cf8fe2b713d9a207767dc4e198477cc2fb64338c6940e27e'), ('scenarios/ipv6-interface-id-units-v1.json', 454, '8f099330086db72ecee290146dac24625ea7ac2ea2da8d995c9a6ab142f68385'), ('scenarios/ldap-administration-v2.json', 924617, '7b746923770f50ec00a7c0e28ef1456de10c324efde6cade39af517737ad3b76'), ('src/ui/tests/eui64.rs', 7154, '588cbb6f1fcdfbb8fceb371747b109a193750ec020ac0449b6c75811c9f7ab63'), ('src/ui/tests/current_catalog.rs', 7660, 'ce1f5dfc5853c11cdf3d6f72fd5394ffe9d345c0ba5917a964b761574902cdf6'), ('src/ui/smoke/eui64.rs', 13929, '61f5234b42a3b42b4399c6db2e1b082b648e8e5fa310facb71f3233bf228b9a6'))

CURRENT_LDAP_SCHEMA_INPUTS = (('src/ldap_schema.rs', 3767, 'dfdfc1f4a2ecb299d5676bc869082e0074d3578804c6075d43ccc36a4a66e163'), ('src/ldap_schema/description.rs', 17376, '5d658c58d9a3f819f25b82187336dd4319d05ec375af1696f0b65d80a88b4edf'), ('src/ldap_schema/registry.rs', 14918, '2b8adcdbc01a9371bd83f00d510a28bdc0cfdf21982234f521c0ca98941677dd'), ('src/ldap_schema/entries.rs', 15378, '0cfbe43b64f2fb24718b6c7df1e184903a77874232a4c7a6deaf3d18313d39a7'), ('src/ldap_schema/loading.rs', 3107, '24ec0a8f85d49211c2691f369f8273952fedfc0b9a97f94047d8aff28dc5cef3'), ('src/ldap_ldif/schema.rs', 6886, 'ab9fc6afb74f8de00e9f2b16687bb9a25bbcbc0621286c0c43bf3f10cca5a314'), ('src/ldap_schema_workspace.rs', 19935, 'aff3acce3fd764412c45a9b2a0b7a7b1031af8e1c9daa3519032976d6fbaafb4'), ('src/ui/ldap_schema_page.rs', 21095, 'a1f636f5ddf0ad4c3ed4272bd3f597a72b26e2921c5656b0283f56e3e6cce925'), ('scenarios/ldap-schema-units-v1.json', 423, '80c55b2318da14f807a8caa8c539e129c28089358de1e1869ab1f36893629ac4'), ('scenarios/history/ldap-utility-before371-v1.json', 604727, '89c5c3863a790bebaa09cc1ca6ff98b517455e3e96c58fb90139ee8251d3e513'), ('src/ui/tests/ldap_schema.rs', 10794, 'e49140a2948cfc9257a7f7e9eff4a376fafde4bd2736a50b7cdbc2d4840ac7de'), ('src/ui/smoke/ldap_schema.rs', 28901, 'c0a34df3ad2503e588d4d29803726b4d0dfc255a7dc81833e6120841975c43d0'), ('src/ldap_schema/filter.rs', 5651, 'a0a4da80e6358caa1fe4684f22852e34deb3867cfcd3e00e89be510252f46736'), ('src/ldap_schema_query_workspace.rs', 28663, '578956905dcd3ae231d64e62dd3f3f3921435e254f44fc84b5d3e51fd1f652d3'), ('src/ui/ldap_schema_query_page.rs', 19997, 'dbf845d411ec80b41fe00a300d060502e566fb77d4c5f65dcef840711bd960ea'), ('scenarios/ldap-schema-query-units-v1.json', 425, '3cac6be96fe6227ea098096561e385e955e159ed87bfb9a94a06b1e909303084'), ('scenarios/history/ldap-utility-before372-v1.json', 701049, 'd1535808d93d90559a7a1b5eee57c1d6bf4bc4248dd13949313b80bdc72d062f'), ('src/ui/tests/ldap_schema_query.rs', 11085, 'e010dd10380722c515793e6789ec382e2c03b2c1325bbb94ce77755114a6abb3'), ('src/ui/smoke/ldap_schema_query.rs', 28228, '7637a22001da6c749d14177bc08d6ca8bbb3af8608d82b237a35b3d0ffed9041'))

CURRENT_LDAP_CONFIGURATION_INPUTS = (('src/ldap_configuration.rs', 13173, 'fd0913a3d4ec0f0f3cd1d203d96147d70252787455b7203d1120a13899126675'), ('src/ldap_ldif/configuration.rs', 7074, '0e05d12007664110820be9fe1b09cb0189dea3043c17dc97dd06b1c3d7934099'), ('src/ui/tests/ldap_configuration.rs', 9428, '91d861ff41110a3e2e40cc88169916521dbb79892bf8b1cd80d2bac590a4e36b'), ('src/ui/smoke/ldap_configuration.rs', 25797, '332bc26a08adb2033d4435582669d25632a0fbb67233f2622269eb6b2e5f2ad2'), ('scenarios/ldap-config-units-v1.json', 442, 'feb76be9adb8369ba44886251f5ecfa89cd8175f1223c7716641deb2e964c2db'), ('scenarios/history/ldap-utility-before370-v1.json', 491118, 'e8b2a678a84e85a5da5cad7e9a03df1df8b9eee4a979244b4ba2c6c7329c4718'))

CURRENT_LDAP_ACCESS_INPUTS = (('src/ldap_access.rs', 18184, 'b460e3578ba10bb0db0c1414bb159d49d2ac0ad087dedd386391ad9f3604b0ce'), ('src/ldap_identity/protected/access.rs', 4091, '16e2f268b8b0bdc2c684a12cb0edd360389dac718bfd7ccabc581be5d57d7229'), ('src/ldap_access_workspace.rs', 30245, '1a558bbe92ef9c0ea9b182af4acbaf496341056ec7535597fb7a76e33e68f2ef'), ('src/ui/ldap_access_page.rs', 32023, '2730391d1b15e05263c9ffd92f3819dced16981f8ce82337c1255bd2cffb55fa'), ('src/ui/tests/ldap_access.rs', 9014, '8fffe1970db241d3a3f30bdd76f7eae3b5c46c48ec63f345a7563a43084b7a14'), ('scenarios/ldap-access-units-v1.json', 434, 'b476fed8f9b001e5edf920ec90d1038210aaf60286b2305f8f4935414da4fcff'), ('scenarios/history/ldap-utility-before369-v1.json', 380809, 'eeaa8818c5dc870b6e375640c9da65067c4172bef0b0c1c4c5147c48f5601000'), ('src/ui/smoke/ldap_access.rs', 24760, '5e5869ce93fdcfb2069e5978ef80749b4ea461a53d6a14e3f869eba4fefd16bb'))

CURRENT_LDAP_PROTECTION_INPUTS = (('src/ldap_protection.rs', 9945, 'f38ad444189ee62d50c6bb8984d9a7ad17e4ae50243832b1b0709395193b75dc'), ('src/ldap_identity/protected.rs', 7239, '32b87f9c8f3661ca3837b246a4c0da9465e8326455ce5728bd0b0392e56dc5f8'), ('src/ldap_utility/password.rs', 8887, 'f69470aa463a16545ecf657809060183fc953b733371a7f0d635ffbe1881968b'), ('src/ldap_protection/client.rs', 4425, '2451c0caa043ad78be737f017dc0f7d24aa1ef41e4832b4f029b31d3df21d20c'), ('src/ldap_password_workspace.rs', 28552, '1b607626ecf741c6b6fccc9f6326cee1324849864d1c69bcbe5a3c440dbea0fe'), ('src/ui/ldap_password_page.rs', 20780, 'f29645eb524ddfef68c43c0ba21c396875330556f868d93ee653dc9c59f2e533'), ('src/ui/tests/ldap_password.rs', 9945, '92f53cf6a97d731fc0f052197f21f195092234c7fe365cecf23e3233df30c81f'), ('src/ui/smoke/ldap_password.rs', 25109, '0735e93aa2e7b12ee1fdb4a84900333c634a6cd9ce381d7c40ae9f2b875a99c8'), ('scenarios/ldap-password-units-v1.json', 428, '55541b5e89c18f8434415ae9d5ca45f93783ff4fd3c27af4ffd3da7395ea71f4'), ('scenarios/history/ldap-utility-before368-v1.json', 321803, '7c737961ed9ca49c060ffb8cc7860802644ed2d916e58eb34a36c0604a5ecd11'))

CURRENT_LDAP_DELETE_INPUTS = (('src/ldap_utility/deletion.rs', 8825, 'b45d9979ac04a62fba3e0daf619d9354488dc6b54348a2fe538aad29868b0720'), ('scenarios/history/ldap-utility-before367-v1.json', 185785, '31093c05080d873388a85ca1ffb8912e795fb5a1203734ecf8001a8d299f9df7'), ('scenarios/ldap-retirement-units-v1.json', 416, '2b9b4aeeccfab20d9f88a3a5d953bd4f11d3e2b0b8615fa2857755d894685a5b'))

CURRENT_LDAP_UTILITY_INPUTS = (('src/ldap_utility.rs', 6039, 'ede9e5b55f3e313fefd749dc2a74d25bb0a755fda268cd427047d4c66454ec0e'), ('src/ldap_identity.rs', 6500, '5ee95af657c733ebb2c1939fc656cf432153c5846ad02baf084d7ffe81c46f2c'), ('src/ldap_ldif/utility_records.rs', 4472, 'bf65f9a5ee4b3cfaaccd92ddc87754158e142ec6b944862232917ead3abcddb4'), ('src/ldap_utility/execution.rs', 6152, 'ba330253e983ce55ce111dc34eb1fa24433246fe7c85d53a4acdcd3e428c7e48'), ('src/ldap_change_workspace/utility.rs', 8722, '86a651092adb36ca6660c624bbcdf87020d6bab932fb04df79dbfe1f3bfdba8a'), ('scenarios/ldap-utility-lessons-v1.json', 763235, '8af9bcb2e5e25e7f0212860e887ba780a45640a770ccecdcc90a0a774896b16e'), ('scenarios/ldap-utility-units-v1.json', 472, '7698e855734d82d70dcba67a830bdc79073f7e980b3b92074477d2ac22e5eb57'))

CURRENT_LDAP_CHANGE_INPUTS = (('src/ldap_ldif.rs', 17657, '9706a6dd0223a0e851678ac4a148f6f014d90900843f29a31b25b4b4e2bd7ab3'), ('src/ldap_changes.rs', 21708, 'e8276197ad9d8ababdc99e4adf338ccbcbe0d3164344037a74e536cb9516cd00'), ('src/ldap_change_workspace.rs', 21750, '0d153f73078965b79fabd632879bbe20c67898eaf50bec5cf05a4a25be8b7948'), ('src/ui/ldap_changes_page.rs', 22742, 'bb78e5bfe5a4ea5d5661801947130a450f84ff702ddef7d89fda1eed9712e032'), ('src/ui/tests/ldap_changes.rs', 20070, '990e65782a33acf6ccee11c84fbf6ec537f4de53a685e26246b83d388a7ea9ae'), ('src/ui/smoke/ldap_changes.rs', 35158, '4b9447fc72256269e10e11bff6128e79b230fb14fea514807e0ed1a1bccafde8'), ('scenarios/ldap-change-lessons-v1.json', 161441, '805545aa059261d6cc64264b1f0d3d01a2d3bf6be34337b4882de6704231c027'), ('scenarios/ldap-change-units-v1.json', 468, '869a00152b5a238954fe6e91d29344276878d9a2e93d125c6b7360021bccdbf7'))

CURRENT_LDAP_QUERY_INPUTS = (('src/ldap_query.rs', 24298, '4b928811f8b5a0f4afdc18021703cc403a00d682f34e0cd3a6d3b0db31071279'), ('src/ldap_client.rs', 19040, '5b63aac93641b85d3a65093451ffbd42f6b54287a0f352a6ca6b653975f8adb6'), ('src/ldap_workspace.rs', 23173, 'dc86b7486f760b55e141d57f6cc33a3e3f7d533f1c6648a680634b7905a2e67d'), ('src/ui/ldap_client_page.rs', 17453, '5b853ee33cbc637b4653302b481e017a6bb66193902c774598fb94ba270d1506'), ('src/ui/tests/ldap_client.rs', 9979, '96c5f8e4392613a763937514f7c58a33dd9260ccd49c1dd3e5a390f0610b504b'), ('src/ui/smoke/ldap_client.rs', 25373, '98360b79377796c735f21a879df8931e9750df99156a9277553187544c04aee5'), ('scenarios/ldap-client-lessons-v1.json', 216948, 'a15e4e1d792940271c7f117a7501742ff55952a8b4529a3839643073fe644fd0'), ('scenarios/ldap-client-units-v1.json', 479, '7becaf87bc1ab3936866f0f9eb97fb9e9ca1a1c716c136ea5ca7965a92f87272'))

CURRENT_AAA_METHODS_INPUTS = (('src/aaa_methods.rs', 9481, '320d64ecf581581ed0a2a4ecfd90e5e50ed210dfdba05769c0963e6ba0f246ef'), ('src/aaa_document.rs', 5380, 'da5561f97f20f7366b0977f1faa229c2f8aeb2f5e3c6ef47f39d0ca19d85342d'), ('src/aaa_workspace.rs', 33759, '56b9f580c1fe8c81e5ece61c5bc55bd1de8bd09a08542b66d023b7db651553c3'), ('src/ui/aaa_access_page.rs', 20487, '1014f2607a46da0db28797b6c22a9d9840bd42470063317715ed1b967f69380d'), ('src/ui/tests/aaa_access.rs', 14306, 'dddd6ad0c844e20f7733f9e03af4c252f4a5a9eaeb2c87841eaad3122e4c6ed8'), ('src/ui/smoke/aaa_access.rs', 24981, 'bac7a9763113ea8be20556e5ac5afe13b02b2d23099d1cc0e67449ca202ca4f7'), ('scenarios/aaa-access-lessons-v1.json', 85355, 'b3f20febcc26214fe81d6cd6251c8326fa8dd18822e7dcf93076d5119d042971'), ('scenarios/aaa-access-units-v1.json', 497, 'cb07addd9deb09efe9712aa2cc0108b1e996d6bf0a43bb22344e774a59fb143f'))

CURRENT_LOCAL_ACCESS_INPUTS = (('src/local_access.rs', 7231, '43615074d5179ca5a2b4c27da71710bd871b1138b05b0e52c75755b10948510b'), ('src/local_access_document.rs', 10251, '1284b8e8a0455ddce62739c8220923eb770df744539e785d3c78256dcd004ace'), ('src/local_access_workspace.rs', 27707, '279100300de5bd4230102a7b8e3ce33a19da3b12a9e54eeb1c5b85b081e60e0a'), ('src/ui/local_access_page.rs', 20073, 'c8cc0fca44c1ffa7c48918593daec37d36dff2c148f9ecca5f449d53d89490c4'), ('src/ui/tests/local_access.rs', 12419, '7f60161537c473519ceab9f3a0cd88d22487ced3aa10aea903f769980ae5561e'), ('src/ui/smoke/local_access.rs', 23804, '4a78dc654c305be62e8ebbccd7cfc2495b0700eef36e379446210f68f8521054'), ('scenarios/local-access-lessons-v1.json', 63872, '59247085c1d169f96d01eb2bfa065cc26411d96f174d2c7ebd29b230d45a61b8'), ('scenarios/local-access-units-v1.json', 503, '815b98a96961b9c4ef3dd518de549cb78ea46a6f4531888c075bd7ed77406900'))

CURRENT_MANAGEMENT_POLICY_INPUTS = (('src/management_policy.rs', 7845, '4b233ea008546891c5d81ddfb6fe1ae1a823516122ea740e5d5d3bfc552dcb08'), ('src/management_path.rs', 7070, 'b1ef772196cc70d268537a19f1092453c59867c37d4e0f3ddf1e27518b6b219b'), ('src/management_document.rs', 17137, '0d8dd8b6dd0f0308e8df920f281c6a16633a529887a353933bd81f6364703184'), ('src/management_workspace.rs', 23375, 'f5a876c46de40caced4ebe5a1aeee01a29c018a3c9eeace232dc083a93a61902'), ('src/ui/management_page.rs', 18718, '50dba8cab412387fcd17cf17344a4de1ad37695b4e38aad534a1b47afeafb08f'), ('src/ui/tests/management.rs', 10447, '815b387b1a44f08b1337dc5850aca526920a551019a671339f41ad1239f0f981'), ('src/ui/smoke/management.rs', 22209, '6c98faefef18db3eaccb4223b442f202dcf272d7c4395ee2eff38daf5cce3c3e'), ('scenarios/management-policy-lessons-v1.json', 45351, '8ac45a8c150f1debc266c703786193620d028fb8beff2bd0c41dbb219d257e84'), ('scenarios/management-policy-units-v1.json', 533, '9130ca7d4dba160e16d5a3ce91bef2e313506a5413a437c93ea1a23d6dea7a7d'))

CURRENT_ARP_INSPECTION_INPUTS = (('src/arp_inspection.rs', 14921, '010c49733612dd3e4437de4a222b1773368c68c1cc37e4e4ea5e13b6c07048f6'), ('src/arp_inspection_workspace.rs', 28339, '42427a10227c997c700dd4aba38877b13754c041a2cd86ce36ab3db0c4d0ff59'), ('src/ui/arp_inspection_page.rs', 28431, 'b0f032d1d1cc92f4c579e6ae501721369d9db2f2ba385c8199ac31405307d279'), ('src/ui/smoke/arp_inspection.rs', 32456, '6d98cdadd8d2b6965f01eef587703472ea7e62c29320fbf6a44c29d6e10a55d2'), ('src/ui/tests/arp_inspection.rs', 12080, '0b6a2d98233acca91fea9fcb062a32e866fcba039fcbc0ecbeee3d46445ca69d'), ('scenarios/arp-inspection-lessons-v1.json', 215124, 'cf10abcff571fa3c760b3298cf683478891621fc2f7db1eabddd128fd0875914'), ('scenarios/arp-inspection-units-v1.json', 520, 'b48ff38bc8372fc7a29c7186745263ed5cd5d927b8b5d04c5c2d6200fb71ee27'))

CURRENT_DHCP_SNOOPING_INPUTS = (('src/dhcp_snooping.rs', 26284, '4383032a91651472286061f4174cc93e20d5b0ddbfab1e6d037b661a7ae55529'), ('src/dhcp_snooping_workspace.rs', 29409, '0ef031cc893d88ae1f9172dc5ad8ecccbc03e3f34c792f7c7618d48cfedf3c40'), ('src/ui/dhcp_snooping_page.rs', 23239, 'e882090cf3501c3019caf87fdbf8d2e96017c5233e72f136a93f0f3c839b762a'), ('src/ui/smoke/dhcp_snooping.rs', 28708, '0401e19d6c673fb0404d0132db3bf581a5dd8b45dcac6a59fc4f6322bc015add'), ('src/ui/tests/dhcp_snooping.rs', 12501, 'd494765ac7dd875a4dc605496592f978bb7ba723cda1df449f34c70691ea87a3'), ('scenarios/dhcp-snooping-lessons-v1.json', 120022, '1dcb0bcfa749e4cc1b3c61e403f9226a4c21edd881907c65369a89dc5b162841'), ('scenarios/dhcp-snooping-units-v1.json', 554, '86b77bc0695dfcc05e0b623397e07ad7d09b85ce824b903e0014d4eb8492c3fa'))

CURRENT_ACCESS_OUTPUTS_INPUTS = (('src/mac_learning/access_outputs.rs', 3896, 'bca68aa33bc3b63ed1e21f8d9ffc78ea79b9bc6cb8851388cbaac0992f888eae'),)

CURRENT_PORT_SECURITY_INPUTS = (('src/port_security.rs', 16721, '318e1f85caa0bd25695907ef05671ac9790f5b371bd95ecc0130d7f41c3df09c'), ('src/port_security_workspace.rs', 29496, '8f4c9486dcafcd00f842fcb6dfc9e10fec867fe30cb8e5a61120011abc4e34cb'), ('src/ui/port_security_page.rs', 21933, '62924cb4a55d7ca1d8a5b1ba8eedae69372b68ad3fd894df7065dca4268296d9'), ('src/ui/smoke/port_security.rs', 32134, 'b7eb3997d49b07f68d4fd88d71ac5febbb4e0f65f59427839975031ff7e04a4c'), ('src/ui/tests/port_security.rs', 13729, 'a8427ae3305a63a46a6e14394a2ec6b26d28b2d944dea9e5c8e96db1ab9320f9'), ('scenarios/port-security-lessons-v1.json', 151163, 'a9ae7657dc6721ffc8f001e705af1371a0d4acfea08817360bd69279eb553029'), ('scenarios/port-security-units-v1.json', 497, '4a4557e2e3829e68ed15ba89ebab794732b8b60e496b57f6a97bf73490a1aa67'))

CURRENT_STATIC_CAM_INPUTS = (('src/static_cam.rs', 5532, '9f5a12cd907a44c2e07e13402c9de3b3cdf3f051ec83458d65cef8c2d76a734b'),)

CURRENT_AGING_INPUTS = (('src/aging.rs', 14595, 'dad8de212b9481f164793bf5cb03dba2de80e3616589985482cd102308e774b0'), ('src/aging_workspace.rs', 16657, 'ac21e186ea8fb06e273f579946560a81b828f8c329c689ee094e8cf88f4c8fc4'), ('src/ui/aging_page.rs', 17884, '7f35c16f443b834ae9421a6f124cd39991307e1bb7ed3787e09acb3459b4013a'), ('src/ui/smoke/aging.rs', 24361, '3e7a2a341acf60899ffbed7d99b48fc62a4945d99285caff6121b417f0b59728'), ('src/ui/tests/aging.rs', 9033, '2cacf3f6fad00a32f852d0566d5085528f3126cee27e0283acb1bc7c384bde6d'), ('scenarios/aging-lessons-v1.json', 35841, '40f6d8391af2b6f2f415700be3e96bd2ca9f8d0f7a792c8acb57cad43dd58200'), ('scenarios/aging-units-v1.json', 528, '75b3910ec8a342fd6212bdda4872f8c4d133ee9bf59d70ed2e3f0d9801d553f0'))

CURRENT_PAM_CONTROL_INPUTS = (('src/pam_control.rs', 9916, '8c710db0a09583679e49f517652b37e1f764ed0b09ad0a11719bf3624a94d28d'), ('src/pam_workspace.rs', 15763, '814e440497cb5349d042cce0c9fcc75a58ccd6f67e239ece031d3607fe91c9d3'), ('src/ui/pam_page.rs', 18446, 'b601da9f669c5e104cc4b6fd1df04e8f430aead38bfbe6bba20b58801439865b'), ('src/ui/smoke/pam_auth.rs', 22700, 'a577bdf8b092b72e36609dcc5fdcbc47e63535d8d2dd83073f480a502d69d4d7'), ('src/ui/tests/pam_auth.rs', 10337, 'b58190e84fc721d3987a0bd9e2317f4e322c97035f88152c51bfd2c4599b7601'), ('scenarios/pam-auth-lessons-v1.json', 8057, 'eba444aabde7571b666983b9364e56d697e2a1e202004627dfe6fac6bc17d02b'), ('scenarios/pam-auth-units-v1.json', 448, '4846c437d9c4c0f276fbe223f5e7ba17c45b3d48aac9cb2a8e5b25c43d4b442d'))

CURRENT_BRIDGE_FOUNDATION_INPUTS = (('src/bridge_fabric.rs', 8971, '84bf5ab998da2472ba3cfc46b4a3427d00874978b83001eb7fad9c3f120566d5'), ('src/bridge_control.rs', 16784, '4db1c3b3eb9d00806f79a7d2f796d4d7fa4c284b5c2c5457fd7f6da1eca21434'), ('src/bridge_host.rs', 10687, 'b1bb681ba2475cf91cc8161a47bca2916c5cf0ba59f8f5800a2ffe6537268cf7'), ('src/ui/tests/bridge_lan.rs', 5613, 'e73bcd7af7814331a30cc1ba5641061a0eb1b16bead942e76812af591511cfd3'), ('scenarios/bridge-lan-lessons-v1.json', 80155, 'cdcded27a07c5042cc57ae86430d895300b82b0716d51c61d8669fefa8ef6213'), ('scenarios/bridge-lan-units-v1.json', 469, '1bd4874f33c5d19fd288fcb0924df5a37353dc3424ab03d86195c31193dbeffb'))

CURRENT_BOND_CONTROL_INPUTS = (('src/bond_control.rs', 23205, 'aa3d027ed71596d5af334376bdabcf89d67b905c80678c176f93f94a6e529cd1'), ('src/bond_host.rs', 8377, '2124444a1f1c4f2dafee509ba53b96dd566d4f08e9e30008675258ea4b7c8573'), ('src/ui/tests/bond_vlan.rs', 5111, '75ee07390d5362a00000564566193459cb3bd860b737422b589920668d4735f0'), ('scenarios/bond-vlan-lessons-v1.json', 85253, '98d2b243e2672afec4ea738c678fd1a974b2cd018bdf1b95c98abbcdb93dc3c8'), ('scenarios/bond-vlan-units-v1.json', 463, '9590030a9e46b1edd4a3b68bc73f84bf9a9e5e907d432d753421831c8d525884'))

CURRENT_VLAN_HOST_INPUTS = (('src/vlan_host.rs', 12022, '8a7382e5366ec37f691999f40c4180feb1108325877eaa32fb9b699235a6009e'), ('src/ui/tests/vlan_lessons.rs', 4300, '8a18c45d95cff24d860e2108f91ce5abc57272be4327eb2037460e9f59896b29'), ('scenarios/linux-vlan-lessons-v1.json', 96756, 'da07f06f95f52e9eae16af01837598b2883161ab36f69ded62e37b4b473c8c74'), ('scenarios/linux-vlan-units-v1.json', 490, '73e913a72740505ed260e537611f3ff8cabee9f0e200d42556208ba80eabaf5c'))

CURRENT_RAID_ASSEMBLY_INPUTS = (('src/raid_assembly.rs', 17962, '406d70ecee7a2356fd1c21fd104b18e7f081850aba8ec80f208aee02ecdbecd2'), ('src/raid_assembly_workspace.rs', 16843, '20190a4eee70dceb341ab2ea9eacb7ed18c14cca559e5447a384560b21d2cd44'), ('src/raid_assembly_source.rs', 1917, 'b82a8edbe4cfabe6bca581050473f00df05437ec18fde03a5dae29a460ae7b4a'), ('src/ui/raid_assembly_page.rs', 21738, '8e789fbc9c60ee6ad0c9283a72ff551bff6af5787955de369c921f31a5c5a236'), ('src/ui/tests/raid_assembly.rs', 11219, '3aa2e670825749b1c7fe09fa0c369b8699442c6a4ccf4d12212084f1726165b5'), ('src/ui/smoke/raid_assembly.rs', 19898, '8a50ab854ebbd1cd2f1f77b25f7395ad32ffb884ed4ddd71782085b6865085d7'), ('scenarios/raid-assembly-lessons-v1.json', 23553, '3afc14cd65e62301f07be59e768f5ae4a9ea32daa004e9f4b71982195980e0ae'), ('scenarios/raid-assembly-units-v1.json', 441, 'ce5f99dd21d3ea46ae502e8a4d666c3922e9e6ac186ed5bf29f27312db09ef86'))

CURRENT_AUTOFS_INPUTS = (('src/autofs.rs', 178, '76e7eb70313f0f802bfe4493235c60ec347dead8cdcc6c618529c698d96658b8'), ('src/autofs/maps.rs', 14390, 'ebcdafd33e90ad213dabd8a3dae6219573bda24a89e4a2a258906ec239e80ab8'), ('src/autofs/runtime.rs', 18679, '0d5f29ac53c14873709656e54d42358ae011b81c97181e78b75a669e0e738dd9'), ('src/autofs_workspace.rs', 29508, '3ba0f4b8467cd92bcf9c71ba636272dcaee84944f89e4c34cf1ab3ca85cd5b42'), ('src/autofs_source.rs', 1806, '90bdeb976e4cad01dddd64acb61a9feee11b6f04411949db505470c0510d3c46'), ('src/ui/autofs_page.rs', 24937, '9ef276cf4a856a459fdd67f33f97cd55a80310a012553dbb1d6023d5ea37ef3e'), ('src/ui/tests/autofs.rs', 12627, 'eda17d6e87b0574e9d5d6f5f5b03e4f32548081d8f254e70c5aa6c96ab2888ea'), ('src/ui/smoke/autofs.rs', 20629, '2bc6491ca52ee04c73d670725c27bfc11d93ef9b06bd404e122a6661fe581a5d'), ('scenarios/autofs-lessons-v1.json', 51013, 'b5ded5c051898e4ab53142e12b6dab00ac48d1a69c41b9f9a76c7e94220f8ca4'), ('scenarios/autofs-units-v1.json', 486, '8cb0e652f95b91e863ce545db0491bbede862efc700c7bd5787c4a871a869fdd'))

CURRENT_LVM_MOUNT_INPUTS = (('src/lvm/mounts.rs', 10719, 'fceee0c706cdb32edbc88b84ba0b68016c2c11dcb6fe6d7978f86aec35bdb530'), ('src/lvm_workspace/mount_flow.rs', 21812, '47222854fc3248166a4b0e4388d95d123fa9d98b048f54f988aa439db14a36be'), ('src/lvm_workspace/mount_session.rs', 6624, '210667ed4e1d3f87bf882abed46308fdbd44a8eb94555e98be581d7b3105698b'), ('src/ui/lvm_workspace/mounts.rs', 7634, 'bb81694a33a34562f31c600bf72230c376f4ba90f4924e82d6b163d0a14cd5f1'), ('src/ui/tests/lvm_mounts.rs', 12852, '6e6b29bbf38b5cd216588c81589092b3110042c680b18dd3b8070845ff6e2617'), ('src/ui/smoke/lvm_provisioning/mounts.rs', 5629, '4e9e8db7c1afd540c2477e7e81b24c5eb5cbddca3c73e3c98c1ed18ed45584f1'), ('scenarios/lvm-mount-lessons-v1.json', 67311, '956fdb4a6bcb7da6016d48d0f50e9606c27d70cdcdda8bc60cd4574d6acfc499'), ('scenarios/lvm-mount-units-v1.json', 475, '995d20f18d926ee3da0e4dd59df810ccca42fa915198b7848e7089bc372190a6'))

CURRENT_LVM_CONFIG_SCAN_INPUTS = (('src/lvm/config_scan.rs', 18162, '9d5fe41977c08eef3a1d425f2f0cf917d2dbc2a9b80c260855a65ac38e528d7e'), ('src/lvm_workspace/config_scan.rs', 23291, '0e359644483e8fa409f5e92524bc460897f3e2c5f71c49cda81144b2ef516291'), ('src/lvm_workspace/config_scan_session.rs', 7653, '2fbda9ee01e94d0f0f076084c16b43231773686252ee415dcacb95179267ef8a'), ('src/ui/lvm_workspace/config_scan.rs', 28457, 'd608484dc83506e16a0411b72d6714f0c887b21d20991c99aa4e8490d98b4432'), ('src/ui/tests/lvm_config_scan.rs', 13569, '9babf7c41d82cb141c94a9605a94e748c563ff2c5ac2e4a4900a9c03f0660383'), ('src/ui/smoke/lvm_provisioning/config_scan.rs', 7064, '502821f9780c2e6056fe9cfccb009c28260904aa025567b871ab7e9ec0db8c04'), ('scenarios/lvm-config-lessons-v1.json', 48477, '87b8b44686c396f989a70b3d3b9284fd4b3476aff0532c64ab028154e23342c9'), ('scenarios/lvm-config-units-v1.json', 457, 'e1bef587982e53968f7cbf3a9a4e67d0916f5198a31e872f786bb1754e279588'))

CURRENT_DEVICE_PATH_MODEL_INPUTS = (('src/lvm/device_path.rs', 4689, 'a1f3172f5f3f5dfae66e4198f601b9eb740219c53d3b0b774b9aa9cd193acfd4'), ('src/lvm/device_reference.rs', 4520, '418d97bcc6abf71005f621ef8e81ea83275c60fc4cd00b27df2e3c818887b830'), ('src/lvm_workspace/device_reference.rs', 6604, '04b2f0848e350bafa8d538b75b45143f5ac67124c2301d03f8fdaf00ac022bdd'), ('src/lvm_workspace/device_reference_flow.rs', 12664, '04fd255de818cc3078f790af1c46066e6ef6b4ccad2e1762e826cf43187e377d'), ('src/lvm_workspace/device_reference_session.rs', 7433, '24732436f0556cf748fae8d47f45ec5bdffa802247bb7491340bd9a5feace3d6'), ('src/ui/lvm_workspace/device_reference.rs', 6954, 'e1bb8e71143d68244b0542f421e317387d108847a620a5982abe69fd06ef670b'), ('src/ui/tests/lvm_device_references.rs', 12222, 'bcd1e0eca403d83c54ff71f7cdea4903551eec2ef427bb971ecf677032966396'), ('src/ui/smoke/lvm_provisioning/references.rs', 7765, '01f34c2ffdfec649c970f61cd0a50e5cc2bcd6a82ccc61617ed2096d8402f536'), ('scenarios/lvm-reference-lessons-v1.json', 64929, 'e5f727c3ba7546445860b8801aa4df852a439ccb6adf455e906bbfc85975cc8d'), ('scenarios/lvm-reference-units-v1.json', 463, '0e4a4ea916b63d26a5d1c69709dbd5e05b9bd7391cb15d03ba0dadb933d0ea56'))

CURRENT_COW_MODEL_INPUTS = (('src/lvm/cow.rs', 17970, '50588a2394695c99011154d80fcadee6eee473168d1efe293b23fd20ca2437b5'), ('src/lvm/snapshot.rs', 15071, 'c5a753fc05d3061e6859d6bfe74209a70bb7047042c8a756ede6ed6c7b888dca'), ('src/lvm_workspace/snapshot.rs', 16765, 'e697ae546494badb9f568ad59879ac4a1a01f2481a49649bcfa7ee5b3e076807'), ('src/lvm_workspace/snapshot_flow.rs', 16769, 'f905fe781c534cd1b4de93eeff2855430b0faba9a927cdd64839e7bde9c9e129'), ('src/ui/lvm_workspace/snapshot.rs', 6363, '59e265b2485e1713bed6973d31f547773f05990789b1b88570c991d865dee35b'), ('src/ui/smoke/lvm_provisioning/cow.rs', 8090, 'c2eafb04dca64e90dcaed61c4f633f0dae087f3f7a234654bff728278095991b'), ('src/ui/tests/lvm_cow.rs', 7940, 'bb2d9b7f754f749f1c1bda2ebed00881f18f2901aaeebe16219b0282fc669cdb'), ('scenarios/lvm-cow-lessons-v1.json', 170801, 'b8d62467ace58f2aa672d0afddddfa8c9c8f45ea4a6f98e92e75b04d1366d1e4'), ('scenarios/lvm-cow-units-v1.json', 446, '70c366fd520243db0ae573ded7570a1bda23c23180d5789c5fe4bbf1b8cdc4ab'))

CURRENT_MBR_LAYOUT_INPUTS = (('src/partition_fs/mbr.rs', 12274, '941441a163febe8937d0f9594885cecead83d409f1199feb016275fce2678437'), ('src/partition_fs_workspace/mbr.rs', 15758, 'bd1da62cd388bcab41c572dcec5c4980ff67455aaabf2b202242264968bfdf3a'), ('src/ui/partition_fs_workspace/mbr.rs', 4360, '0c468bfef200c2fde4b0d7b3d6322fc1b24289a7cf2768ab1dcf36b02d163803'), ('src/ui/smoke/mbr_layouts.rs', 26108, 'ac7bf860762d1c9caf1d7aa7c3f4d4e9ec7b1da710d3efb63c75e2f61c46bf36'), ('src/ui/tests/partition_mbr.rs', 16052, 'edb6c87335a45aadb9e5fb8b8730bda3485dffb0f74f87bdca6b39673e1fd00c'), ('scenarios/mbr-layout-lessons-v1.json', 127637, 'c57800c6e32eead9698c7a9235f40b849403fc9c62f6099270bf9eab11fab6f2'), ('scenarios/mbr-layout-units-v1.json', 534, '4155502a53271e08300ed8fa91418f07bca4b7a863ec6a4d99d7256d8e25995b'))

CURRENT_PV_CAPACITY_INPUTS = (('src/lvm/pv_capacity.rs', 5586, 'c92301b913486243b55e9325df71b93da220ceb3e6cd876adea9ecb093d85c20'), ('src/lvm_workspace/pv_capacity.rs', 3484, '90bbaf66ef762429a218d9c16625b824f6c84e4309cdad7ee47ad7bc52d04054'), ('src/ui/tests/lvm_pv_capacity.rs', 14095, '541d8c59de486f6fced6c0a2a009eb4119721b4287427a80107887a133228b8f'), ('scenarios/lvm-pv-capacity-lessons-v1.json', 52566, '6f0227d16a4c8c8b0d14cab0430ac4862ce2653d3b7481bbcb6a7274f01d4379'), ('scenarios/lvm-pv-capacity-units-v1.json', 461, '4ee570a6587618e976e572c6566bb1ea0b668172cc4791ff333a69f4119f9d5f'))

CURRENT_LVM_RENAME_INPUTS = (('src/lvm/rename.rs', 8376, '743393628f766e12dc4fc8b640db0f95292b2a71a5959aac81ec073089dff3f4'), ('src/lvm_workspace/rename.rs', 10794, '9cee29414801b005015b106af837fabd6e3fc91a3322f916beee7b23af238b3c'), ('src/ui/tests/lvm_rename.rs', 12614, '1378ffe7c04cb097d94a66ea6f2a3b7856d0cfd3071a8f8e82b1f578f245599c'), ('scenarios/lvm-rename-lessons-v1.json', 51951, 'ce9e2929d7f9fd8ef8c5e9235bee1a344a1ffc1a16984be29373ee8e3269dedc'), ('scenarios/lvm-rename-units-v1.json', 443, '83ed693c7e460adacef49bf795f7fe0e237d6b4782eeb67dd82273e287f643b9'))

CURRENT_LVM_ACTIVATION_INPUTS = (('src/lvm/activation.rs', 6078, '4f29f957e5192e3ecedd08070d075afe209326625caf09e941e80f97c139bf0c'), ('src/lvm_workspace/activation.rs', 5193, '8a65ab60ca0acd1c7c6e5a283874ec5957341b4c79c197ed2b4d5a50c5a3af62'), ('src/ui/tests/lvm_activation.rs', 11128, '0c00cd4256d8ee2ef32145876d604536226b17df904e769d70ccdc93761e0007'), ('scenarios/lvm-activation-lessons-v1.json', 45111, '4376ff7e53998fa26c9fabfd9d6b401ba697091966d5cbe3f9085d277c2bca89'), ('scenarios/lvm-activation-units-v1.json', 434, '2c752e2d31e2b93abf4bb140cee95270c491725c547802b87d30cd06db1d920d'))

CURRENT_LVM_RETIREMENT_INPUTS = (('src/lvm/retirement.rs', 4501, 'ff51a04cc2fcc30c86b97a0c197117f53eeb78b6701015d0550321ad9e937d35'), ('src/lvm_workspace/retirement.rs', 9504, '60f33e61c55b3eb086adf025e0d6d61dfcaf39bf42c6d12b9a14c3e2df4b7b37'), ('src/ui/tests/lvm_retirement.rs', 10532, '6fe15396c7eff4923a3827912b6407972fd522052bb11aeb06a755343ab8bb98'), ('scenarios/lvm-retirement-lessons-v1.json', 45903, '4ee94cf8037d42badce284c5f1cf978a274bb63688117bc7cea71c8030ae22de'), ('scenarios/lvm-retirement-units-v1.json', 431, '236416ff458a29db67bf0b22b6b602e8a2c65574c82ef8c4d5e879e8811fe46a'))

CURRENT_LVM_PROVISIONING_INPUTS = (('src/lvm/provision.rs', 12014, 'b8c51013dbbbe2fd478b2a91cee29854687d48723610bfcaa0fafb8975fc1247'), ('src/lvm_workspace/provision.rs', 8791, '2a2e12146113a65a88ccd6392ad2e3c9bfa84d1e2d941919837218b287fe0fa1'), ('src/ui/smoke/lvm_provisioning.rs', 38584, 'b4dda6c35d863de04a07bde2992881da4a310fe2e0190f355243dbd0fc10ae9a'), ('src/ui/tests/lvm_provisioning.rs', 9739, '29e737970ec968ebb939b001900371c14f46095755a5a3a57bc680f6574a9e88'), ('scenarios/lvm-provisioning-lessons-v1.json', 56239, 'ce9bd717c5dbf2ed6aa2454ceeddd3ffe75416f16a7506f59ac13b8242749131'), ('scenarios/lvm-provisioning-units-v1.json', 478, '9866b063f73a40c8e66822dcc512bdc09bf9138ecdd723c84750f6c6d5a46466'))

CURRENT_SWAP_POLICY_INPUTS = (('src/storage/swap.rs', 13046, '6a6196329cb7c8bfed2faace3512d528e147d90252cf0543af8e630deaa03063'), ('src/storage_workspace/swap.rs', 14359, '5c09223a99c31a0fd3b9045fff103a135bd81dd9426ca6b5e0e7fe73489135a8'), ('src/ui/smoke/swap_policy.rs', 24975, 'f83c17af4c77411d1f8c436c4b63bcde24d57103303f4fefdc561f6178f6421d'), ('src/ui/tests/storage_swap.rs', 13350, '5c31b2265f84cc7b7a6b7f315d9c9f116944d234aaf0c1277420522dd27db766'), ('scenarios/swap-policy-lessons-v1.json', 39876, 'b8ca77691e9b1a51c610c7d9aedbff5709ad7368fb71bc4899295b41561d6331'), ('scenarios/swap-policy-units-v1.json', 494, '881d90542c396f121c5cf84cd10051efd9853b9b5e843a4c19bd467159b20117'))

CURRENT_HOST_IPV6_INPUTS = (('src/linux_host/ipv6_host.rs', 22800, '55da1bc10718ad840a9e6a36c138189c7f21280119126bf36995e70441e3e30e'), ('src/pack/collection_budget.rs', 927, '3bd2f22d98bdfa64fef649b70db746950f4fe02d0ea1f6d2f447f6742820d6e5'), ('src/ui/smoke/host_lessons/ipv6.rs', 5062, '3d835e3030c1732bdb1df0faca0257cde4ba06f4053fcd3f958d49c6804a5eee'), ('scenarios/host-ipv6-lessons-v1.json', 117433, '7e437e02cfd8a157fef16ae8a174b37898f358bf02bb1818496e22dfea95290c'), ('scenarios/host-ipv6-units-v1.json', 504, 'f297d69b5e0cb971046f5a4d7552a588d8c4c9662968adca4f6305928158230c'))

CURRENT_RPM_METADATA_INPUTS = (('src/rpm_packages/metadata.rs', 2086, '44ce8c3bf97a144cf27e63e389901c241534018c8d071f1bc42bce20f1bf8de3'), ('src/ui/smoke/rpm_metadata.rs', 15066, 'e4fd0e8ecfd8b5960559de807724f26844c835cb4bb93017df581665a45263e0'), ('scenarios/rpm-metadata-lessons-v1.json', 74816, '188f66505a9e9cd31562a014db57f985a0037e467bee12d4877ebc14574c3caf'), ('scenarios/rpm-metadata-units-v1.json', 499, '5e0296256028fed96206ef2c23bfff9cb6b5a733dddf1fc31531aa8f359483d7'))

CURRENT_LOG_TRIAGE_INPUTS = (('src/ui/tests/log_triage.rs', 7149, '2074e62d9c9fe1e4837513f268b4b043f04e51c547394180e1327ca75c542fdc'), ('scenarios/shell-log-triage-lessons-v1.json', 38426, '522c62a4fa42790b1d7fb5e76de96d4cf13a33ef63779f75bdc8802a6751c11f'), ('scenarios/shell-log-triage-units-v1.json', 503, '82c67d5b2c85c7cbe8a5d843c2a674a89c6ff909784605c5b7760dbf2322f640'))

CURRENT_KERNEL_NAMES_INPUTS = (('src/ui/tests/kernel_names.rs', 9498, '39f3fccf7e333057c35a34693534f38fd5a01a15dc2b3d4e042fca13af8261a6'), ('scenarios/kernel-module-names-lessons-v1.json', 19661, '4aae368c4682ea69e3468008d86d6d9884d72ac7f64a3d1671bde0733d0ad7e0'), ('scenarios/kernel-module-names-units-v1.json', 391, '423b6648c357c375cbf3d37dea0e6236c2ccfa18084c4c35e51512c599dee678'))

CURRENT_KERNEL_MODULE_INPUTS = (('src/kernel_modules.rs', 30564, 'ac14a6c96620cfac88b55840912ff92874293278e89dc50c14e9fd75a70b16d5'), ('src/kernel_module_workspace.rs', 26968, '8b04630767eea307afa960de87796e632910fa47caa668cabdac55fb6b533c82'), ('src/kernel_module_source.rs', 1534, 'ad6c8a91781611a7c13f690a40cdd58c233b92e66c2c437a25ba15673ec5ef12'), ('src/ui/kernel_module_page.rs', 22776, '4c8e2032fe46ee51d6abe713a645cb4b1588b6ccfeb7433c1d2e21a7a66c683d'), ('src/ui/smoke/kernel_modules.rs', 25547, '2f565d33bffdd6e1b623121cfe2610c3130d2a9d72955950124b6f2643033e2d'), ('src/ui/tests/kernel_modules.rs', 8610, 'ddd533cc43637c42096a6005c0384a17c8695421c35b43d765b8486cca227f04'), ('scenarios/kernel-module-lessons-v1.json', 52069, 'a294100b417a473960f22b2396b9042b4862b3dc68a3187375a27bcf2c989634'), ('scenarios/kernel-module-units-v1.json', 828, '1d9d328b3eee13f3932170fe099c0c6ab4a9ac11a334aabd5ac14faf32ec95b7'))

CURRENT_SIZE_SEARCH_INPUTS = (('src/filesystem/size_predicate.rs', 4107, 'f79b9938e56b39ffddc48d90f74b2ff390954a13c90e7b80a60b12205b266e88'), ('src/ui/tests/find_size.rs', 8745, 'dae64fffcb43d6c693dd3b7fef9e2fd7e1800e6b5267306c04a0e4780c8d1090'), ('scenarios/find-size-lessons-v1.json', 60791, 'abc588e1abc539d74e1fa5aafe04d5588211f4bd774aabc6e7ebc4f2cc51fe6d'), ('scenarios/find-size-units-v1.json', 457, '5921b0d0ed0d3c6ab7ce7ad2f0a8487b2f9ee1a639b3bb9575e1f04e6f43979a'))

CURRENT_RECURSIVE_TREE_INPUTS = (('src/filesystem/recursive.rs', 14128, 'cccf8fe4824accd8d3c994b64c4b04f8ca4acddb03fc7532c4c32eb6d707770e'), ('src/recursive_tree_workspace.rs', 23728, 'a9c95e1784dd236455f1a368f695b6460b7a338f749514de05f70957f85abce0'), ('src/recursive_tree_source.rs', 1803, 'de31f84c8e13be9f8a786cec33ae1d1635e2341876f9b10fa7e6cb9de5570d74'), ('src/ui/recursive_tree_page.rs', 16768, '0bd54bae113a7878e231ed03856eedd12962d5265fd92f413e9a9b6c7266375c'), ('src/ui/tests/recursive_tree.rs', 10550, '0ac615b2dcdda01d7cf4c2eeab8930fe0a5bcafb0a6b7761eff4233db4f8f213'), ('src/ui/smoke/recursive_tree.rs', 24629, '3a3970b7a052602e1040802065faa6c2ab949285d5c60f289a41655886e5f264'), ('scenarios/recursive-tree-lessons-v1.json', 46602, '9c0c5822ead3e37fd329885d1adeef97ad5a92dafe1284269ceb255249088501'), ('scenarios/recursive-tree-units-v1.json', 470, 'a0faea49dd9674ab07e126f35cc32b8fdeb2b9b588d9c9bd14096097a823af5b'))

CURRENT_TRANSPORT_INPUTS = (('src/transport_core.rs', 18764, 'ae52437675ab968f0d926f1e0078fc4c8037c5ffb6b7c01b47bc41cbbf1972db'), ('src/transport_workspace.rs', 25130, '85c0ab49f47f1cd813afa2ba3b9864d9febf9c9a483e4bb2f6a67d85a3cec0bc'), ('src/ui/transport_page.rs', 24567, 'd606ace3b42711e55fb1f2d974255c617d6124f26e002bfeb171200837921905'), ('src/ui/smoke/transport.rs', 11583, 'c57162839e0ba685dd22d56c5694ade665551b9b2b1076c586925f57c6673421'), ('src/ui/tests/transport.rs', 10361, '2981e1f1b6c906b2d77b6692339c7aef478ce35c514a9de04cf7b8fd826ccc49'), ('scenarios/transport-lessons-v1.json', 65068, '24c404cf2446d58aee627764471e8196ac5086b342ca766adb27ff7818ca2c28'), ('scenarios/transport-units-v1.json', 542, '5101f25b163a2682bf01ca88fda03589190036364a6bfd726cbe0c3866471777'))

CURRENT_CONTENT_BACKUP_LESSON_INPUTS = (('src/ui/tests/content_backups.rs', 7698, '16509627459480935ac9e3f7b2ae99e8c2e017689dc8bb24877782db9e51ff4d'), ('scenarios/shell-content-backup-lessons-v1.json', 48176, '97d4f0b4891266b3e2baa5afaff7e5b4e899043157963075dfcefc5bc94e3954'), ('scenarios/shell-content-backup-units-v1.json', 446, 'c3f7e9798e78513a21ee44d5bb15788a6c785ed87ea2f6bccf8aa3d2919daf76'))

CURRENT_DIRECTORY_POLICY_WORKSPACE_INPUTS = (('src/directory_policy_workspace.rs', 20863, 'ce245374de438ef172f657d1e9cc8ce7eb63673eb14e39d82d9233f279051caf'), ('src/directory_policy_source.rs', 1552, 'fe000d2244a40415e92054aeb2d0bd4650432fd509ba08a8d621e32b1976ae5d'), ('src/ui/directory_policy_page.rs', 18457, '63f6743f765603b470641e4e1b95f7f14acd801e2f27d26dd1ed35c0a04555b2'), ('src/ui/tests/directory_policy.rs', 10975, 'f419ec2d4fa2ff447a8deb0437bf83cc3c567d7f8c0f00d5e86c04efbe6e3adc'), ('src/ui/smoke/directory_policy.rs', 17445, 'b46ff520673d090c6e96c22efd26ab3007a1a3584a905636b4074101f5b10963'), ('scenarios/sticky-directory-lessons-v1.json', 42438, 'd7e49b8af26bbd69eed6bc825ae51d4a39f8615c61223d045c624f70138d12ab'), ('scenarios/sticky-directory-units-v1.json', 459, '371201eb4f738feba7ce502af9d5bc39fedba3b1a5473d0fae4dfb9de8817b6b'))

CURRENT_DIRECTORY_POLICY_CORE_INPUTS = (('src/filesystem/directory_policy.rs', 25460, '0cc2697cb5e4902084b419a8f0e5874a8e7712cf3b311d506941700a3d91014e'),)

CURRENT_WHEREIS_ADAPTER_INPUTS = (('src/filesystem/whereis_deployment.rs', 14301, '588965b43f586d9818990b2b6cbff2cbe436169bc3b2bd0388f15cce1ed1ce9d'), ('src/filesystem/discovery_policy.rs', 2912, '3114a1307a0c6481c093a80c53ff94356089065eab727c661417f87e3c655390'), ('src/ui/tests/whereis.rs', 6798, '70718849aa770f2bdee5eaa3d08af4c3ac16e4dbb25bae2d23e920586725c7ea'), ('scenarios/whereis-lessons-v1.json', 47499, '2eb7bb7e031abb70d6e69011ef58fbd9b8e7f22d116581348cda8d16ee7e1c1a'), ('scenarios/whereis-units-v1.json', 496, 'ea54bb69e87d4f242d3f763a5aa164021d23bdbf041028d8e66569fb1e8c43c6'))

CURRENT_WHEREIS_CORE_INPUTS = (('src/whereis.rs', 14831, '19d096c8b9583974347e35e4ef2cb68a8277c469157fb403005b77ae426e11ce'),)

CURRENT_FILE_INDEX_WORKSPACE_INPUTS = (('src/file_index_workspace.rs', 42454, 'ca2b0797020e468ee91084c75717bd6ee2bcc0a8ce7763a336bfe83fc86461c8'), ('src/file_index_source.rs', 6064, '059546c52faa6b683f32fc8eb3bc831bfe31170fc84a2c0095749052a8fd1060'), ('src/ui/file_index_page.rs', 21819, '3095338f3175257c22f44b9784b996362221bc9542f249599a79e430485d8383'), ('src/ui/smoke/file_index.rs', 16437, '64d7d4ef390a998fa327fc324ee093b136438da541bd89c43cce1ed280e23e99'), ('src/ui/tests/file_index.rs', 10192, '00752a7e88cdf89b27c0397e6cf794d7ca4110622f3c9d847300a4ad1e2d5120'), ('scenarios/file-index-lessons-v1.json', 41903, 'ea7efe462815b1746a59e02d92164c225af935225763f5f194e62e105ccdd1f1'), ('scenarios/file-index-units-v1.json', 458, 'd5138796fb1daf8af3de20807e8dcad2d50ae077e2832211795077953e4c98b0'))

CURRENT_FILE_INDEX_CORE_INPUTS = (('src/file_index.rs', 29874, '372dfa1f825e835e94bb768ec783764cafbf92157118e0292b664810607d6ff5'),)

CURRENT_COMMAND_LOCATION_LESSON_INPUTS = (('src/ui/smoke/command_location.rs', 21137, '28c92c371bedd7cc4f90a5b66a9d5fae012245cfc7b9af6bad0b62bcea6adca7'), ('scenarios/command-location-lessons-v1.json', 26114, '63b179ee4f20ba0920958cb236c8f62a03200f42e63860cdd8aee5c2f2d74bb2'), ('scenarios/command-location-units-v1.json', 484, '65a4124017d728dabe4f7a7645c56be8f6b714eaea5a4fe51ff692f92ebb7885'))

CURRENT_COMMAND_LOCATION_BINDING_INPUTS = (('src/filesystem/command_location_policy.rs', 10853, '971c9a4e9d63e7c712cc438a67b79bae81266d219d044be7bd3784679243fde8'), ('src/command_location_workspace.rs', 10692, 'cc99ef76164db862b5d05bd3eaba3bf0a553554815256004164a137ce3677b58'))

CURRENT_COMMAND_LOCATION_CORE_INPUTS = (('src/command_location.rs', 15040, 'fdbdcae12998051a81c61546e04fbc76d33e8de0e4e6088b2eef82412ff6a666'),)

CURRENT_SKELETON_ADAPTER_INPUTS = (('src/accounts_template.rs', 13891, '92cd386b7cc919233351854e0662bca558789f491d0be98797718cfec9fe46be'), ('src/accounts_template_workspace.rs', 11601, 'd767d5365ccb70f0c2ec34df27307e4143422e1afc1451398de588d56e7f8306'), ('src/ui/accounts_template_page.rs', 8462, 'e8a6e1b60049700e68d7d2b9a0f0cd53638ca582eb2e2f805a588b93a722e980'), ('src/ui/tests/accounts_template.rs', 10493, '3277ce3ee625d39e12b4a012a3113e3937628fac5fd31bb720b9645117f7d38a'), ('src/ui/smoke/accounts_template.rs', 19813, '6b542789783f7c67198beb0f82fa47bf2a954f69c8093e922a771e81baf9765c'), ('scenarios/account-skeleton-lessons-v1.json', 69889, '9b2d895dbc93de5d4b0848b4440452c5e25411156ed0f257253d323ead9d971a'), ('scenarios/account-skeleton-units-v1.json', 463, 'bb2dab9582e73cea78f9c52d4fc871b1e4f03466d674555909a11054986ada05'))

CURRENT_SKELETON_CORE_INPUTS = (('src/accounts/skeletons.rs', 5303, 'a1eab9948f7cf3c774dfcdb158e9966e4ba7414dcc3bc90dc1331f7ace05ae7e'),)

CURRENT_PATH_LOOKUP_INPUTS = (('src/filesystem/path_lookup.rs', 16789, 'b9830cc147d667372b710218abd416c1c4439de5ff7a0ba8ee7b93d9f114080b'), ('src/environment_workspace/path_lookup.rs', 11677, '59844ac397f3496a76e6550b2ec1562a42e021270be34e659fb953bf7333e950'), ('src/ui/environment_lookup.rs', 5910, 'd7c22342091371b24f27ee533c5c9f4b63ef5f4da256c7c9557fab1ad42c6db9'), ('src/ui/tests/path_lookup.rs', 6857, 'ed48d775f13e51ef8e94a70f57d7a378006de30e41a22138487a7bcefcdbbab9'), ('scenarios/path-lookup-lessons-v1.json', 79959, '247f830f0980d57078ef891f117582c8579d459af956fd711f5186aae0bcfcf9'), ('scenarios/path-lookup-units-v1.json', 435, 'b5dacc7bd29b99158451c9d72cbcdc69932f1f9fed865e4f431dc320c8ea2d1c'))

CURRENT_LISTENER_INPUTS = (('src/processes/listeners.rs', 11929, 'd67bbaec42172ad274362b3b8b99c694ca97150867881f1161e59688ca7dda98'), ('src/ui/smoke/listener_audit.rs', 15216, 'b7506e9e89330f01238d79bf17abc8419ba479fd0ead5e6a1b40e75c5681b675'), ('src/ui/tests/listener_audit.rs', 6902, '0116420e7215c1b357791b8b2eb7ebd63367ba2708e95cbabd93284b66e4ea6c'), ('scenarios/listener-audit-lessons-v1.json', 27099, '7dc45efdb60430211950cbe8515c9ab5ac252799f9323dfd8dcaa40e06c899d4'), ('scenarios/listener-audit-units-v1.json', 474, '6b58bccdf817c90bd92636dcbd35f5526ca5110233d651f379f389cce40bcdf9'))

CURRENT_MANAGED_CONFIG_INPUTS = (('scenarios/shell-managed-config-lessons-v1.json', 23367, 'aa1367ad74006bfb41fd41c133956485fdcfd0fd122353cc063729f0f375eeaa'), ('scenarios/shell-managed-config-units-v1.json', 518, '99d054b720d3777d5b4655cee5115db547cff6694a27ea4e31072b5314841985'), ('src/ui/tests/managed_config.rs', 6206, '912f85f212fcb560b28a2fc7cffcf96c3ef7506699fff4249ae69a7f78b7a2ec'))

CURRENT_FHS_CONTENT_INPUTS = (('scenarios/fhs-lessons-v1.json', 55390, 'fe4dd20ae310ce02e31aa2baf207f9dd63d91b8ca3e4396ec0c597c4331ca708'), ('scenarios/fhs-units-v1.json', 521, '131d41ec15b99034ccac76f3c28b2c0645946c9135c0b9123052cfebf0a61271'))

CURRENT_SHARED_LIBRARY_INPUTS = (('src/shared_libraries.rs', 29984, 'd0f3f570a1c93b24126dfce244d17f47dab241b01d458d1a01300b66699973db'), ('src/shared_libraries/tests.rs', 16304, '2d19aad7a8db7d050a7debe878704e23218ea626768ccc80205621cd8ba1476b'), ('src/shared_libraries_workspace.rs', 48656, '74cfce68e6d3e18483e2eb533fa0917fe14fd5c3aea32b533720f48eb734116a'), ('src/shared_libraries_workspace/binding.rs', 14284, '8cfc5847f75429b5cbcbddb54957f897160e050a22085ab0ac5a2188fa9dc3bf'), ('src/shared_libraries_workspace/tests.rs', 33924, 'c1c6822f05f17c0ccd64a6fb5abfb112f3ff746c9842113938a3aa0691df5e17'), ('src/ui/shared_libraries_page.rs', 50232, '4327006ca8f66fe96e5dd7c11dc4ec53988bc648af1019ecb71fb95d62e7afcb'), ('src/ui/smoke/shared_libraries_lessons.rs', 45638, '378f38d724bdf0d2c5095cf46d193542e56575b1293891296db7f0284d41b5ad'), ('src/ui/tests/shared_libraries_lessons_native.rs', 23395, '32e8eb95e7ebf53d31e876c6a64387ed415f2627ed4721d876ac301c6640d507'), ('scenarios/shared-library-lessons-v1.json', 97855, '46c684c7fe1eb00e76144c45e5ba980841a0e8ddcb4e93cf8edd9f7fd1e94b04'), ('scenarios/shared-library-units-v1.json', 543, 'ec006c3c3684036c66e88b748663eb537e5eabd59220cb0729265ce9bd48da22'))

CURRENT_PRESENTATION_PAINT_INPUTS = (('src/ui/lesson_presentation/paint.rs', 18616, '4d5b01c263facbbd5b09f6bb176d29403cdeaeda8b92a8ef78f2ff0d61238d28'),)

CURRENT_COURSE_METADATA_INPUTS = (('src/course/metadata.rs', 60357, 'c33ef4bfbd5c073e33597738eb35c18d60e0deaee6e81d5cff7fcc9695a0f829'),)

CURRENT_ASSOCIATIVE_LESSON_INPUTS = (('scenarios/shell-associative-lessons-v1.json', 41692, '76da42b5ab67d2e8c168abcae87b49155a081e1c117079d373d4e05ac1bd191a'), ('scenarios/shell-associative-units-v1.json', 491, '019c26f834ddb9152b607195cf514614f0fa84f12ddceaa88735a1a77f396585'))

CURRENT_ASSOCIATIVE_INPUTS = (('src/shell/associative.rs', 9643, 'f769ab9348c5c1fe3d9f98cb8afbd099ad4813b4454912d45482c044da788670'),)

CURRENT_RAID1_INPUTS = (('src/raid1.rs', 37116, '1f48031533c88d0874a731a0dd91b38c78070c215a58498b8a0dfb9ac93b6ac2'), ('src/raid1_workspace.rs', 28113, 'e99f0adac5e8dd1119e582835e7a6f230d4ca3040c16e7379f9eefdcc6c57ab3'), ('src/ui/raid1_page.rs', 25625, 'c77ef813709fe4705154c1bcf9d8f5a17065b586e4aa458c3e24ee499b73057e'), ('src/ui/tests/raid1_lessons.rs', 12405, '6aabb7360d13cde19209ec51aaa720f6182df41a60b9532b9d9b09d4471924c1'), ('src/ui/smoke/raid1_lessons.rs', 24768, '05e733942373bf780680c18c401c918ba2199d73a112b28c8ea2eea0afc711fd'), ('scenarios/raid1-lessons-v1.json', 107858, 'f9b23c0bc52d715ed3013ffeb8e31f7183bca7eb15b97bf90a39ad6bdbd3c05f'), ('scenarios/raid1-units-v1.json', 468, '3fe429962b051ebf887aff42163b8833538625c28223a56d82489f25307f04bd'))

CURRENT_DEBIAN_CONFFILE_INPUTS = (('src/debian_packages/conffiles.rs', 12060, 'f14edfdae90367fd69fbbab3d88f78d79f64596b0986eb05f297dde38d686d03'), ('src/debian_packages_workspace/conffiles.rs', 11904, '75dd8a005b863f2573562cab96839e733a2208f0a7e9ea7669b5553d191c0c9d'), ('src/ui/debian_conffiles.rs', 21030, '2c1ed0905fb4bc437a6c89ffe5d591d3aa1a5b6646173a3ebd3e05aad86fd5c0'), ('src/ui/smoke/conffile_lessons.rs', 18747, '2a672f831a579dc064971bb43fee6a7037c4bbc643b33dad46005958967444d2'), ('scenarios/debian-conffile-lessons-v1.json', 89072, 'd1733d2fe68c8967e2f174b2b9cdd71a197f5af89adff2799edb125d71dd5f4c'), ('scenarios/debian-conffile-units-v1.json', 493, '1f5239f450f7136abe68069458693be2686a8516fffd59d4144ce6507d96b9ce'))

CURRENT_SHELL_FILE_LIFECYCLE_LESSON_INPUTS = (('scenarios/shell-file-lifecycle-lessons-v1.json', 49361, '6d66d3910b581d2e488b7aa5dcd69885bfdcd12d6e7ecb49f4c3d6e739449ac9'), ('scenarios/shell-file-lifecycle-units-v1.json', 525, '8e847a881485147ab498970f52e5f5b75f89adb539fb246604dc487228f32d20'))

CURRENT_SHELL_FILE_LIFECYCLE_INPUTS = (('src/shell/file_lifecycle.rs', 5495, '1f2bd1547cefac0d20d691399ec0a2dd689784268a3d5092e4b2ed2bb18a30cf'),)

CURRENT_SHELL_STARTUP_INPUTS = (('src/shell/startup.rs', 13877, '85d7f30220497e60c2f6e7c054895d55e4f5113ee73623ad76cda029ba199613'), ('src/startup_workspace.rs', 26269, '0f6d48ecdc6b853d89f15ab2f7ec1c7962e66f89289e6430ddd25f654a860e79'), ('src/ui/startup_page.rs', 26837, '68684e368dc8e0a1e0e9283663952929d6af988458eca74f88cb7f693e6ff25e'), ('src/ui/tests/shell_startup.rs', 10226, '1fd4862e1e8c5692f26e7576a60c2378e99c49bfbdc634f21057c5f92c61746d'), ('src/ui/smoke/startup_files_lessons.rs', 19261, '5f5f6e69168814e17ea47f0ad0fb9d840530c1052a7983eeb764ffeec0dd547b'), ('scenarios/shell-startup-lessons-v1.json', 60192, 'c78a123f6d6255c2701089cd060eee68a3c42072898c419e88f28fad58b54957'), ('scenarios/shell-startup-units-v1.json', 480, '683a5b2eb1f637a4a144d4ba87b6f5876fb073966c8eaa80a0ca204e3c5d5614'))

CURRENT_FS_INTEGRITY_INPUTS = (('src/fs_integrity.rs', 19424, '92a7512e26df52fe36b403772e5c75d029126e02d0f7756dd42152478bc469bd'), ('src/fs_integrity_terminal.rs', 21203, '21fd19a1d935c71e4c6a8c5458a03088467bab9746562475e336620cc4f83ec7'), ('src/fs_integrity_workspace.rs', 29881, 'eccace249c170aa8088d23334389e60245fd0784301581d43b262b113e9751c9'), ('src/ui/fs_integrity_page.rs', 22779, '0e3896afd46379050d0d8b7ec2d6cad8c517db77b16d9027a1b8a0dbe8721b3e'), ('src/ui/tests/fs_integrity.rs', 11063, 'db0b5b56cd088af9268e5f400de8b39154cdd51b5d618a8b1951853ff5477480'), ('src/ui/smoke/fs_integrity_lessons.rs', 18990, 'a85e8790f1b5b9120b180599ca39fb8d322e22e0d5a232692ffe17bbabecd044'), ('scenarios/fs-integrity-lessons-v1.json', 122523, '9bf1968886a46096b95442ba0095729877fc6d7d802c6300088e668530495703'), ('scenarios/fs-integrity-units-v1.json', 531, 'fa1eee168c6d44a4657ca01f189ce7f85709c6bcb7ad32c0444c37df5bb2bdc5'))

CURRENT_NM_DHCP_INPUTS = (('src/nm_dhcp.rs', 3472, 'e3e93b49c194a694c3f508e6c1abfb03607a220c07a704aa4ff3405c19059198'), ('src/ui/tests/nm_dhcp.rs', 5600, '3076ace6139843c3caa42478c23ff9a988af038801c1f2e8eda2ab9c4b347881'), ('scenarios/linux-nm-dhcp-lessons-v1.json', 77507, '494ac0cd19c55e5ed38b272151e1ec1383b14b5f4c53955168c707779218215a'), ('scenarios/linux-nm-dhcp-units-v1.json', 470, 'cb8d8b1170862fd43831ae4a7a74ca9c1fd44f9d4d13aea560a3fd9ecb746bd5'))

CURRENT_DEBIAN_CONFLICT_INPUTS = (('src/ui/smoke/debian_conflicts.rs', 18245, 'b5ae29591a53a2e0d849dc48a5d74df310ba255047232609f10d42e95b634cb3'), ('src/ui/tests/debian_conflicts.rs', 6836, '972dbf788139060a1ba73f8d654acf161313aa15513dfec1e01049e12c34478d'), ('scenarios/debian-conflict-lessons-v1.json', 55114, '4323529805f729e9e251a92f027c56e79abcdc30479b7d0014f35c2e555642b7'), ('scenarios/debian-conflict-units-v1.json', 517, '9a1a11955f95bb9adbce37b37958f4ee0e35639a5912ae2f8722e3adc4cc33d2'))

CURRENT_USER_MOUNTS_INPUTS = (('src/user_mounts.rs', 23919, '963fb9f2b88793c17aa9627d6df78e8f4df0a99e8a97bc9fcacc59c7760ec0ff'), ('src/user_mounts_workspace.rs', 36186, '9fa9f9074ef9d0475a56fe098feae3f46029745da3064544b32bac6d1062d9f3'), ('src/ui/user_mount_page.rs', 33156, 'a234817e122ffea969b1e7d3428cf72f96731744588417f14f4ba755021b0278'), ('src/ui/tests/user_mount_lessons.rs', 10279, 'afc85584a629b1ff4c9350d3b777d04f12e64263aa5872bfd91c2078e98f3f10'), ('src/ui/smoke/user_mounts.rs', 15534, '3a810b238791687fd766a9f41d76f17978b422e4474b35135e518603185e3cbd'), ('scenarios/user-mount-lessons-v1.json', 236998, 'f0588142c40d6cbf0781740072f2e22d2339d2613ac2b89d1879f2f3c7af9130'), ('scenarios/user-mount-units-v1.json', 477, 'bce0bfe3249ba5a24c1ac1ae6b5964cad6e14fcb98174a8419d951540fa6fc7d'), ('src/ui/smoke/reopen/user_mounts.rs', 16964, '16271da8b603cd54ff8cffe08c1f9cfdf85801b8b3a5467c3389dd58ffb87131'))

CURRENT_SHELL_ENVIRONMENT_INPUTS = (('src/shell/environment.rs', 12597, '03f5b37b44e298b0bbceaa62b533219cc7d3047f3410519b073ba64ca84d36e7'), ('src/environment_workspace.rs', 25956, '20d05f0fbc3810ab1825d3d65118027a9ba26e8c36547c6ac984ab172338edb6'), ('src/ui/environment_page.rs', 37551, '5204617207de972fe11a0388b091de66761a759ab052339464710ba1b7b87015'), ('src/ui/tests/environment_lessons.rs', 9138, '3a94512099e2c5d86a917f0c1364848f85b812005a02754e3038fddac02ac73d'), ('scenarios/environment-lessons-v1.json', 111412, 'a0c7e6fe6c91e01c59ee915802453b5334f235a23fd2b8b760e890e8cab67753'), ('scenarios/environment-units-v1.json', 523, '77d9655deb9553fe7f7131c1b3d82a2dedc96b4b35c6b375bd2c6ee7499850e9'), ('src/ui/smoke/reopen/environment.rs', 20044, '28cde63076e57a2593738e6af8a4cf7cb9363e888d4d8731c46b01b1c0605680'))

CURRENT_SHELL_TRACING_INPUTS = (('src/shell/tracing.rs', 5917, '08e2c531ff0533bb6c56c18271cafc69c05aa4beb8b089f4f6e8caa642baeea2'), ('scenarios/shell-tracing-lessons-v1.json', 29904, '2ee8ab8c240d810df3cc3c65b848d1c37675182286afe5af512e5f165993d2b5'), ('scenarios/shell-tracing-units-v1.json', 454, 'fe723168e80b699e8f5cef4b6f2701bc60b27601f0a337c31eb0a37c096aceb9'), ('src/ui/tests/tracing_lessons.rs', 9325, '1adefcfc33c760f303115286ef6b3ae990a56825068317d2ff67ffa60311a4e2'))

CURRENT_DEBIAN_SOURCE_UI_INPUTS = (('src/ui/debian_sources_page.rs', 24024, 'aa8330cf82002952b30899b42e5f32d388b17f191fd2d1a95812dea15d947c75'), ('src/ui/tests/debian_sources.rs', 17453, '124edd3942d8d072d6e29c6f966904762aa21f2003718abcc4f68aea0a1c29ac'), ('src/ui/smoke/debian_sources.rs', 20448, '2ace5ba25be3f3aa9a21ddfdb695aa01a1891eeca2e2c3bef14d4ff1deb6a15b'))

CURRENT_DEBIAN_SOURCE_CONTENT_INPUTS = (('scenarios/debian-sources-lessons-v1.json', 43095, '080e648542d723314d669556330dd7db54a3d6d1a1a739a8be4990b59093ab41'), ('scenarios/debian-sources-units-v1.json', 476, 'af1d2ce8b99ed0bbd0623817f27f8a5078126de355958d7c80272fd170e6f4ff'))

CURRENT_DEBIAN_SOURCE_WORKSPACE_INPUTS = (('src/debian_sources_workspace.rs', 21151, '5cfe9bf9b1000b496784f5c6228f297f4165b6c2f80f28a499c60b794af55dca'),)

CURRENT_DEBIAN_SOURCE_SELECTION_INPUTS = (('src/debian_sources.rs', 12553, '2df49b89164fa3fbc210be5dc45bd38a44780172e7baf46656b6b69f0a6b4fa8'), ('src/debian_source_cache.rs', 13285, '06f973926d1b32b15b7708c15a0e5c5f587eb5bebe05294bf5a4cc3858ce886b'))

CURRENT_DEBIAN_CONTENT_UI_INPUTS = (('src/ui/tests/debian_contents.rs', 13343, 'fda8a82e77aff52fe91e8eb9e619380892ee5fed292eaac276f4fc29884d9591'), ('src/ui/smoke/debian_contents.rs', 20874, '2c8ccf9a49849a555e8781a0506e6b2ff03d8222d1d1200673d9df524b5ef0e8'))

CURRENT_DEBIAN_CONTENT_INPUTS = (('scenarios/debian-contents-lessons-v1.json', 24128, '45819f71bcd92c7a545e35cb38a97008fe3d04ed7b37a01b2974d2f2dc4b6cd8'), ('scenarios/debian-contents-units-v1.json', 437, 'a9a18548f8996cf62303e6ea389a2a678a529c651239d416f5d3f9a5a47ee05a'))

CURRENT_DEBIAN_CONTENTS_INPUTS = (('src/debian_contents.rs', 6516, '9dd2fe8d5f9beb66791b0143fba967c9f3d2140a0bf0e5e163b8aa58cd5bfd96'),)

CURRENT_RESOLVER_UI_INPUTS = (('src/ui/tests/resolver_search.rs', 15565, '93cd5e7c0641c6003f4b52d3fadbb37e23d5fdf6b4863393ee03bcfff01b2d19'), ('src/ui/smoke/resolver_search.rs', 17611, 'e8d7b96e77d55cd2386a14656df7483c928abd87d8b118643ed1bedf4a3be1fa'))

CURRENT_RESOLVER_CONTENT_INPUTS = (('scenarios/resolver-search-lessons-v1.json', 24523, '92b4c66851d44b911406993a34274ecf91370bdee0451c535401888ed1cfeacf'), ('scenarios/resolver-search-units-v1.json', 507, '05ae4d22bdc87d8477101849b87a4b9825f0cb7ea07502df56d9201707306a69'))

CURRENT_RESOLVER_SEARCH_INPUTS = (('src/resolver_search.rs', 9325, 'e63ebb2707e2265959ca01a9c84a3108b3f5bceb1f4871faa34db4b4b3dfc178'),)

CURRENT_CLIENT_NAME_LESSONS_INPUTS = (('src/ui/client_names_page.rs', 40239, 'd55c1766aa815b27787b266accc8a785d6a6517bdfd058f286ff0d175c0bcbb2'), ('src/ui/tests/client_names.rs', 15062, 'c8769744f984779c61aa8bd2ad54a9f4cb07220f8bca22aa8a41afea24b643a9'), ('src/ui/smoke/client_names.rs', 34033, '48f4fe64e4fb56e27a3d5da3f05647fadd52a417856f764e31b1ee687618d429'), ('scenarios/client-name-lessons-v1.json', 28910, 'c6567b55dc559e5e3cba64f15f88604feac202ad046ce2be80737f238ddec226'), ('scenarios/client-name-units-v1.json', 479, 'b7d4622b80b8a99f629df51994618ae3a711655042ab0d3cf8895d851638fe56'))

CURRENT_CLIENT_NAMES_WORKSPACE_INPUTS = (('src/client_names_workspace.rs', 36598, '30f4ac3b052e5100b61a476c2395623d59826d44ab8edce41e530e5380c79750'),)

CURRENT_CLIENT_NAMES_INPUTS = (('src/client_names.rs', 19278, '74c625f6c3af7ac5da568682e4bfbf58654491cf4ef1532486e1ff06adb217b6'),)

CURRENT_REMOVABLE_LESSON_INPUTS = (('src/ui/optional_mount_page.rs', 22966, '572a44cc038c6fa3403f2f6ef91964ccdc8fe9c17a44b8fc34cadd54e8b9377f'), ('src/ui/tests/optional_mount.rs', 13456, 'ef7c39e50bb49ba0d2b75f5e3d7cf9f43b8b0be2faf5f6bb4e6f4bcac86df9b9'), ('src/ui/smoke/optional_mount.rs', 14619, '47e78b2757f602b2eb64d9375bb0a9e79b519c4bc6ae86b40efe86309a8028ec'), ('scenarios/removable-mount-lessons-v1.json', 18713, '6e2fc1bc5abf57d97ee12a81cf1007eaa0ab10c286f340c6fa7f905bfcead677'), ('scenarios/removable-mount-units-v1.json', 465, 'b48775fa6929d2531f9f2070bd31902746180b9c179fad69e2f0c154a7846e98'))

CURRENT_REMOVABLE_WORKSPACE_INPUTS = (('src/removable_workspace.rs', 26617, '3c4826b7c63eeb72381a7a22aaad664101e958a77a3206c1a4bf9493e99bc039'),)

CURRENT_REMOVABLE_MOUNT_INPUTS = (('src/removable_mount.rs', 10511, '23663d2b7e8afaffc79a548ab6ffad00d694202c82721cb8dddcd9f7b687ae07'),)

CURRENT_NM_WORKSPACE_INPUTS = (('src/nm_workspace.rs', 51119, '72e140f5c5cfd3e39f06268fde9c5db6edbb70f77faf993384ddd24d96e344c3'), ('src/ui/nm_page.rs', 36749, '61456349d97371f49b4abd7c283e593a0d85191212656420097ce415f179a6d9'), ('src/ui/tests/nm_lessons.rs', 13449, 'b8f529fa00fc2d8f6310cf3dfa89b73c477a10c0ee0241151f1800bea7e94d10'), ('src/ui/smoke/nm_lessons.rs', 37594, '1eabd693e3add01944bcfe64e02c7914f974d447c00857a562a124d7f868e384'), ('scenarios/linux-nm-lessons-v1.json', 77350, '2fafc0fc451474c83ae4fa9b8379989836fd7c74ff8fb26d8373869959f5cbc6'), ('scenarios/linux-nm-units-v1.json', 544, '8c4d13037f6bec2b2bd76cf41b60a045537a6abc9c8f8cd8ae0e0f93066ed4bf'))

CURRENT_NM_POLICY_INPUTS = (('src/nm_policy.rs', 38258, 'f97f41daa823cfd9942ec5e9df281df3a11b27c4e9523228d50afe7954204932'),)

CURRENT_NM_PROFILE_INPUTS = (('src/nm_profiles.rs', 24002, '2e4ab2848c569c92ae437e2b15560283f3f586fa2017f2c95f8a27a69b50d468'),)

CURRENT_DHCP_WORKSPACE_INPUTS = (('src/dhcp_server_workspace.rs', 36303, '017b78fe426a75379332b79b801d2caa4911c988f3c453e88ddcac8e6e9a4943'), ('src/ui/dhcp_page.rs', 23065, '1b94bef435ec4dfbdd5b32eada047408bd346244b613aa9255251b3ae0ae9be1'), ('src/ui/tests/dhcp_config_lessons.rs', 15172, 'c4fcf8dac952ca4bf581cd80d235e24e652d99d2020a645853d280689a748174'), ('src/ui/smoke/dhcp_config_lessons.rs', 11243, 'ca6943fecacfe19c259df996c3a87e1ee6da841b6c8373cd65de6c732c6f0b3c'), ('scenarios/linux-dhcp-lessons-v1.json', 80217, '6da240109c2303fb4a4ac41d1284ea6c0769a19bb752401e24b55e24282172f2'), ('scenarios/linux-dhcp-units-v1.json', 520, 'd143fe3c609f9eb64130feda7dc492aab934ca6557b5424e280329aab6dcd15d'))

CURRENT_DHCP_CONFIG_INPUTS = (('src/dhcp_config.rs', 19713, '7865fa3a5a961deadeaa25a0d1f5f712d18cf590300047fecd78ab4192d787e4'),)

CURRENT_BACKUP_SELECTION_INPUTS = (('scenarios/file-backup-selection-v1.json', 65942, '4fdb1daa5929e3c1de8e0002427144899e9698185cc4de4d1ff9376fb5cdca49'), ('scenarios/backup-selection-units-v1.json', 516, 'f26e3bc9f29276211c5de5632548676b4847d75c2ceaeb4fa36d560029e6eb93'), ('src/ui/file_backup_workspace/selection_tests.rs', 5109, '28d0620591c1c7e9a743f2a0bd0c4a0bb69ec9396a6493339d5bad10a92cce87'), ('src/ui/tests/backup_selection.rs', 7670, '63ceec540b5a53cd68bea650381b75db6c7ecce58d80daa8abd354028deca81c'), ('src/ui/smoke/backup_selection.rs', 12528, '8018f952efd05e4d2531161a4f6ea72839163649054888e0467a005928f99476'))

CURRENT_QOS_UI_INPUTS = (('src/ui/qos_page.rs', 24913, '0e9c939f5211128b3748be131a46433959abd4fb5f0df112e95c12ccddec924d'), ('src/ui/tests/qos_lessons.rs', 9181, 'f44db0510f97956eef046e1f36fa1ccd4f59263136c375d72cab28b27201e478'), ('src/ui/smoke/qos_lessons.rs', 14116, 'f52268919ba67531b6c7c0f4b07fbce922fc87b321fc69149a46a9a3d21db392'))

CURRENT_QOS_AUTHORED_INPUTS = (('scenarios/qos-lessons-v1.json', 55918, '95d47961e4b699ced2ff980fb84cbadd9b6740ae5dc88f62f7e9ebceddb37cee'), ('scenarios/qos-units-v1.json', 475, 'e77be3789ff52ded77306a12afe5c1580865896878453b94336e567b10db9b61'))

CURRENT_QOS_FOUNDATION_INPUTS = (('src/qos.rs', 11262, '57f34acf8f12834b68be05641c29e07017015d3833d72d17967475e90fd8c183'), ('src/qos_workspace.rs', 21386, '59eb25453e1f397f77da41c9858b8acd19ca884c3e91591031cb7654d3673325'))

CURRENT_STARTUP_LESSONS_INPUTS = (('scenarios/linux-startup-lessons-v1.json', 93930, '0c63c15c5d835a975606110f5041a6071741cb4c32f17dde0f213fba0d5b9e3a'), ('scenarios/linux-startup-units-v1.json', 519, '0feaf260e0114cd42909f269f0be735de4f193ccc8bdb6f98c78e22738936270'), ('src/ui/tests/startup_lessons.rs', 8199, '29a81c6c614eb18312161ecb1989bd86874ddf85dbe003a4f88c3760dada2a00'), ('src/ui/smoke/startup_lessons.rs', 10666, '309d5e9bf4d6a35f7519701ee18311f0010d2d2246b717efdbaf9e0d0e3cce30'))

CURRENT_LINUX_LOGGING_INPUTS = (('src/linux_logging.rs', 9698, 'a110f3ced69d567c0cbe0de086837131369d779f656613b0dcf4c7aa37f44b9f'), ('src/linux_logging_workspace.rs', 9792, 'a01d87036415f435fab950c8fb6b1dcca38be2a16326f69ffa0fd46d867a2b81'), ('src/ui/linux_logging_page.rs', 14333, '0c3b169de6d926502b150da2ae31599d0e42433936cd21c6ce726872041720f8'), ('src/ui/tests/linux_logging_lessons.rs', 11716, 'a177a981393f7f648db3f0f109bb796a9113a87ced5f81661e2ab8e14b096824'), ('src/ui/smoke/linux_logging_lessons.rs', 10762, '69caee06dd750182e67026ac3d466685e4085d2559aef4a27a2d050d497d5b43'), ('scenarios/linux-logging-lessons-v1.json', 32220, '2f9a8cc9d3dc38aa49b765e1c777e7bcea9caef91e5f1e85175fda7727f46d1b'), ('scenarios/linux-logging-units-v1.json', 524, 'aba2f579dfbc0baf4f966b8645d6f62a0d655424dda83836d2ea3d93ca88e6b3'))

CURRENT_DEBUG_RECOVERY_INPUTS = (('scenarios/shell-debug-recovery-v1.json', 37785, 'bc5647171d95d54816dacd1777603c87a6a6d0174bfee6509a6c694a96e9adae'), ('scenarios/shell-debug-recovery-units-v1.json', 483, '1b7ae8c65de166a7868e6c1874b30ac0b05329ab3d63cd90c11b3be3187fee80'), ('src/ui/smoke/debug_recovery.rs', 39152, '3291716a79abd82a5ee20bb44e5022a7ead1c304f297eb3ad34dff92f3cb9739'), ('src/ui/tests/debug_recovery.rs', 9540, '8ea666ccf00627b9d382d2b52338c3991c88c5df87dc58e6c2a3e93b0ff2aa58'))

CURRENT_HOST_FOUNDATION_INPUTS = (('src/linux_host.rs', 31959, '919e6fcff761b7e2d6f987671034601e58005d4efea0cc988b40072e70d35d16'),)

CURRENT_REPOSITORY_LESSONS_INPUTS = (('src/repository_workspace.rs', 21640, '99389c28f6e85990c0b3bd1023131e93baede09bf8c8f23bcec6565b47017a72'), ('src/ui/repository_page.rs', 26497, '06a953a7708d16e4c23c1b4bbe2ebe544197c1603d834b7cb8b2ab2011dc1a04'), ('src/ui/tests/repository_lessons.rs', 11929, '7cb006a4744df77c9652fb979e93bb532b738eb1a44f6c961b917e274abb0794'), ('src/ui/smoke/repository_lessons.rs', 13379, '8a6cd35591d7510bfc6120e357a28333ce4fa8cd84960c112bbfdfa29568aaf4'), ('scenarios/repository-lessons-v1.json', 140116, 'e404d95cc0d3c0a8f40bbd1cb2bc98337db8d56c61f37ab405ffd0a4dae93231'), ('scenarios/repository-units-v1.json', 538, '0f11bf879173dc6bc386be77d3807e16ce77a092014af66c6115427f095e3a3d'))

CURRENT_CAPACITY_LESSONS_INPUTS = (('scenarios/capacity-lessons-v1.json', 28213, '733e65d8859c4567576b3079c90e87ded0dcdd194bc9fb3175b1d7e9483dfac3'), ('scenarios/capacity-units-v1.json', 524, '48f123cce009008c9299595967124d9c7b51c851a5939d4ece3e2bd31c509f69'), ('src/capacity_workspace.rs', 14577, 'bd48bb8169c026927c2f16d627a45c2ea789288a87db3b58b68bf6ff027d1788'), ('src/ui/capacity_page.rs', 18182, 'bc7ac1318037d1786826cb602a86464682c966cd887ed8a764120ddeed0c0159'), ('src/ui/smoke/capacity_lessons.rs', 11563, '145fea99b4fbbba381e8b8544dbe8d7b68a2b3be1e5932248842b3968756883f'), ('src/ui/tests/capacity_lessons.rs', 11836, '49288be55e856e3345b5c98dabf56a43852419827e337ba5006c04f084c322b6'))

CURRENT_LINUX_FOUNDATIONS_INPUTS = (('src/rpm_repositories.rs', 18886, '9def063c2f1ccad271d8ea77dfed22e44be8f68aa6e634155b50e0fa2f1f0381'), ('src/fs_capacity.rs', 14379, '67c50926ae9e36445461e6b49fbca0b6b0a20d1241842bfb7f914482fe0d1bc2'))

class ShellEreReferenceGuards(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='dlms-ere24-guards-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.raw = (PROJECT / c.FIXTURE).read_bytes()
        self.fixture = c.strict(self.raw)
        self.put(c.FIXTURE, self.raw)

    def put(self, name, raw):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw.encode() if isinstance(raw, str) else raw)
        return path

    def expected(self):
        chain = self.root / '.build-artifacts/toolchains/rustup/toolchains/stable-x86_64-unknown-linux-gnu'
        paths = [str(self.root / n) for n in c.SOURCES] + [str(chain / 'bin/rustc'),
            str(chain / 'lib/rustlib/multirust-channel-manifest.toml'),
            str(self.root / '.build-artifacts/toolchains/READY.json'),
            str(Path(sys.executable).resolve(strict=True)), str(self.root / c.PROBE),
            str(c.shared.GREP), str(c.shared.MANUAL)]
        hashes = {str(self.root / n): sha for n, sha in c.CORE_PINS.items()}
        hashes[str(self.root / c.FIXTURE)] = c.FIXTURE_SHA
        hashes[str(self.root / c.PROBE_SOURCE)] = c.PROBE_SOURCE_SHA
        pins = [{'path': p, 'bytes': 1, 'sha256': hashes.get(p, '0' * 64)} for p in paths]
        with patch.object(c.safe, 'source_identity', return_value='a' * 40):
            return c.control(self.root, pins)

    def probe(self, expected=None):
        expected = expected or self.expected()
        outcomes = []
        for row in self.fixture['cases']:
            predicted = row['expected']
            outcomes.append({'id': row['id'], 'row_sha256': c.digest(c.canonical(row)),
                'outcome': {**predicted, 'files': {'/work/' + k: v for k, v in predicted['files'].items()},
                            'error': None}})
        return {'identity': c.identity(self.root, expected), 'outcomes': outcomes,
                'GNU_exclusions': self.fixture['vm_only']}

    def test_exact24_null_predictions_and_all_six_reason_only_rows(self):
        self.assertEqual(c.fixed(self.root), self.fixture)
        self.assertEqual(len(c.IDS), 24)
        self.assertEqual(len(self.fixture['vm_only']), 6)
        for row in self.fixture['cases']:
            self.assertIsNone(row['observed'])
        for field, value in [('cases', list(reversed(self.fixture['cases']))),
                             ('host_execution', 'fabricated'), ('vm_only', [])]:
            self.put(c.FIXTURE, c.canonical({**self.fixture, field: value}))
            with patch.object(c.safe, 'capture_command') as child:
                with self.assertRaises(c.safe.OracleError): c.fixed(self.root)
                child.assert_not_called()

    def test_contained_leaf_files_allow_literal_dash_but_refuse_escape_and_large_data(self):
        self.assertEqual(c.files({'-': 'one\n', '-records': 'two\n'}), {'-': 'one\n', '-records': 'two\n'})
        for value in [{'.': ''}, {'..': ''}, {'/absolute': ''}, {'a/b': ''}, {'bad\nname': ''},
                      {'a': '\0'}, {'a': 'x' * 4097}, {'a': '', 'b': '', 'c': ''}, {'a': 1}]:
            with self.assertRaises(c.safe.OracleError): c.files(value)

    def test_only_exact_supported_fixed_argv_reaches_held_gnu(self):
        rows = self.fixture['cases']
        for row in rows:
            self.assertEqual(c.host_argv('/proc/self/fd/71', row, rows), ['/proc/self/fd/71', *row['argv'][1:]])
        for bad in [dict(rows[0], argv=['sh', '-c', 'anything']),
                    dict(rows[0], stdin='changed'), self.fixture['vm_only'][0]]:
            with self.assertRaises(c.safe.OracleError): c.host_argv('/proc/self/fd/71', bad, rows)
        for binary in ['/bin/sh', '/usr/bin/grep', '/proc/self/fd/not-a-number']:
            with self.assertRaises(c.safe.OracleError): c.host_argv(binary, rows[0], rows)

    def test_actual_probe_identity_every_input_and_all_outcomes_required(self):
        expected = self.expected()
        good = self.probe(expected)
        self.assertEqual(len(c.parse_probe(c.canonical(good), self.root, self.fixture, expected)), 24)
        for field, value in [('engine_revision', 13), ('compiled_sources', {}), ('manifest_dir', '/foreign')]:
            bad = copy.deepcopy(good); bad['identity'][field] = value
            with self.assertRaises(c.safe.OracleError): c.parse_probe(c.canonical(bad), self.root, self.fixture, expected)
        for outcomes in [good['outcomes'][:-1], list(reversed(good['outcomes'])), good['outcomes'] + good['outcomes'][:1]]:
            with self.assertRaises(c.safe.OracleError): c.parse_probe(c.canonical({**good, 'outcomes': outcomes}), self.root, self.fixture, expected)
        for field, value in [('id', 'foreign'), ('row_sha256', '0' * 64)]:
            bad = copy.deepcopy(good); bad['outcomes'][0][field] = value
            with self.assertRaises(c.safe.OracleError): c.parse_probe(c.canonical(bad), self.root, self.fixture, expected)

    def test_probe_status_and_virtual_files_are_typed_bounded_and_errors_do_not_pass(self):
        expected = self.expected(); good = self.probe(expected)
        for field, value in [('status', True), ('files', {'/etc/personal': ''}), ('files', []),
                             ('stdout', 1), ('stdout', 'x' * 16385), ('error', 1)]:
            bad = copy.deepcopy(good); bad['outcomes'][0]['outcome'][field] = value
            with self.assertRaises(c.safe.OracleError): c.parse_probe(c.canonical(bad), self.root, self.fixture, expected)
        bad = copy.deepcopy(good); bad['outcomes'][0]['outcome']['error'] = 'Honest simulator limitation'
        actual = c.parse_probe(c.canonical(bad), self.root, self.fixture, expected)[0]['outcome']
        self.assertFalse(c.compared(self.fixture['cases'][0], actual,
            {k: self.fixture['cases'][0]['expected'][k] for k in ('stdout', 'stderr', 'status')}, {})['passed'])

    def test_full_stdout_stderr_status_files_and_prediction_are_independently_compared(self):
        row = self.fixture['cases'][-1]
        expected = row['expected']
        vm = {**expected, 'files': {'/work/' + k: v for k, v in expected['files'].items()}, 'error': None}
        host = {k: expected[k] for k in ('stdout', 'stderr', 'status')}
        self.assertTrue(c.compared(row, vm, host, row['files'])['passed'])
        for k, v in [('stdout', 'wrong'), ('stderr', 'failure'), ('status', 1)]:
            self.assertFalse(c.compared(row, vm, {**host, k: v}, row['files'])['passed'])
        self.assertFalse(c.compared(row, vm, host, {})['passed'])
        bothwrong = {**vm, 'stdout': 'same wrong text'}
        self.assertFalse(c.compared(row, bothwrong, {**host, 'stdout': 'same wrong text'}, row['files'])['passed'])

    def test_control_requires_all_exact_typed_source_compiler_executable_pins(self):
        good = self.expected(); self.assertEqual(len(c.check_control(self.root, good)), len(good['pins']))
        for pins in [good['pins'][:-1], good['pins'] + good['pins'][:1], [dict(good['pins'][0], bytes=True)] + good['pins'][1:]]:
            with self.assertRaises(c.safe.OracleError): c.check_control(self.root, {**good, 'pins': pins})
        for field, value in [('compiled_sources', {}), ('host_rows', True), ('main_head', 'wrong'), ('root', '/other')]:
            with self.assertRaises(c.safe.OracleError): c.check_control(self.root, {**good, field: value})
        bad = copy.deepcopy(good); bad['pins'][0]['sha256'] = 'f' * 64
        with self.assertRaises(c.safe.OracleError): c.check_control(self.root, bad)

    def test_exact_root_approval_and_duplicate_keys_are_required(self):
        expected = self.expected()
        c.shared.approve(c.canonical({**expected, 'root_reviewed': True}), expected)
        for bad in [expected, {**expected, 'root_reviewed': 1}, {**expected, 'root_reviewed': True, 'extra': 1}]:
            with self.assertRaises(c.safe.OracleError): c.shared.approve(c.canonical(bad), expected)
        with self.assertRaises(c.safe.OracleError): c.strict(b'{"status":0,"status":1}')

    def test_prepare_launches_no_children_and_missing_approval_refuses_capture(self):
        expected = self.expected()
        with patch.object(c.shared, 'root_guard', return_value=(91,)), \
             patch.object(c, 'source_pins', return_value=expected['pins']), \
             patch.object(c, 'control', return_value=expected), patch.object(c.safe, 'capture_command') as child:
            with contextlib.redirect_stdout(io.StringIO()): self.assertEqual(c.run(self.root, 'prepare'), 0)
            child.assert_not_called()
            proposed = self.root / c.BASE / (c.CAPTURE + '-PROPOSED.json')
            self.assertIs(c.strict(proposed.read_bytes())['root_reviewed'], False)
            self.assertFalse((self.root / c.BASE / c.CAPTURE).exists())
            with self.assertRaises(FileNotFoundError): c.run(self.root, 'capture')
            child.assert_not_called()
            with self.assertRaises(FileExistsError): c.run(self.root, 'prepare')

    def test_prior_capture_is_retained_and_never_overwritten(self):
        out = self.root / c.BASE / c.CAPTURE; out.mkdir(parents=True)
        diagnostic = out / 'diagnostic'; diagnostic.write_bytes(b'preserve')
        expected = self.expected()
        with patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c, 'source_pins', return_value=expected['pins']), \
             patch.object(c, 'control', return_value=expected), patch.object(c.safe, 'capture_command') as child:
            with self.assertRaises(c.safe.OracleError): c.run(self.root, 'prepare')
            child.assert_not_called()
        self.assertEqual(diagnostic.read_bytes(), b'preserve')

    def test_cli_has_no_source_binary_pattern_or_path_selector(self):
        with patch.object(c, 'run', return_value=0) as run:
            self.assertEqual(c.main(['--mode', 'prepare']), 0)
            for args in [['--mode', 'capture', '--source', 'anything'], ['--mode', 'capture', '--capture-id', 'other'],
                         ['--mode', 'capture', '--binary', '/bin/sh'], ['--mode', 'unknown']]:
                with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit): c.main(args)
            self.assertEqual(run.call_count, 1)

    def test_gnu_version_matches_exact_held_descriptor_status_and_framing(self):
        good = {'stdout': '71 (GNU grep) 3.12\nLicense information\n', 'stderr': '', 'status': 0}
        c.shared.require_gnu_version(good, '/proc/self/fd/71')
        for bad in [dict(good, status=True), dict(good, status=1), dict(good, stderr='warning'),
                    dict(good, stdout='grep (GNU grep) 3.12\n'), dict(good, stdout='72 (GNU grep) 3.12\n'),
                    dict(good, stdout='71 (GNU grep) 3.12\r\n')]:
            with self.assertRaises(c.safe.OracleError): c.shared.require_gnu_version(bad, '/proc/self/fd/71')

    def test_child_timeout_retains_unknown_status_and_no_completion(self):
        out = self.root / 'capture'; out.mkdir()
        with patch.object(c.safe, 'capture_command', side_effect=c.safe.OracleError('SYNTHETIC timeout')):
            with self.assertRaises(c.safe.OracleError):
                c.child(out, 'case0', ['/proc/self/fd/71'], b'', {}, c.safe.Budget(c.CAP), (), c.time.monotonic())
        status = c.strict((out / 'case0/STATUS.json').read_bytes())
        self.assertIs(status['complete'], False); self.assertIsNone(status['status'])
        self.assertFalse((out / 'COMPLETE').exists())

    def test_snapshot_preserves_final_lf_utf8_and_refuses_extra_symlink_or_hardalias(self):
        work = self.root / 'work'; work.mkdir()
        (work / '-').write_bytes('é\r\nlast'.encode())
        self.assertEqual(c.snapshot(work, {'-': 'é\r\nlast'}), {'-': 'é\r\nlast'})
        (work / 'extra').write_bytes(b'')
        with self.assertRaises(c.safe.OracleError): c.snapshot(work, {'-': 'é\r\nlast'})
        (work / 'extra').unlink(); (work / '-').unlink()
        (self.root / 'other').write_bytes(b'private synthetic')
        (work / '-').symlink_to(self.root / 'other')
        with self.assertRaises(OSError): c.snapshot(work, {'-': ''})
        (work / '-').unlink()
        import os
        os.link(self.root / 'other', work / '-')
        with self.assertRaises(c.safe.OracleError): c.snapshot(work, {'-': ''})

    def test_raw_result_complete_status_and_integer_agreement_are_required(self):
        folder = self.root / 'probe'; folder.mkdir()
        (folder / 'stdout.raw').write_bytes(b'data\n'); (folder / 'stderr.raw').write_bytes(b'')
        (folder / 'RESULT.json').write_bytes(c.canonical({'stdout': 'data\n', 'stderr': '', 'status': 0}))
        (folder / 'STATUS.json').write_bytes(c.canonical({'complete': True, 'status': 0}))
        self.assertEqual(c.retained_result(self.root, 'probe')['stdout'], 'data\n')
        for value in [{'complete': False, 'status': 0}, {'complete': True, 'status': True},
                      {'complete': True, 'status': 1}, {'complete': True, 'status': 0, 'extra': 1}]:
            (folder / 'STATUS.json').write_bytes(c.canonical(value))
            with self.assertRaises(c.safe.OracleError): c.retained_result(self.root, 'probe')
        (folder / 'STATUS.json').write_bytes(c.canonical({'complete': True, 'status': 0}))
        (folder / 'stdout.raw').write_bytes(b'changed\n')
        with self.assertRaises(c.safe.OracleError): c.retained_result(self.root, 'probe')

    def test_evidence_inventory_refuses_extra_files_and_symbolic_components(self):
        out = self.root / 'out'; out.mkdir(); (out / 'file').write_bytes(b'data')
        c.artifact_inventory(out, {'file'})
        with self.assertRaises(c.safe.OracleError): c.artifact_inventory(out, set())
        (out / 'link').symlink_to(self.root / c.FIXTURE)
        with self.assertRaises(c.safe.OracleError): c.artifact_inventory(out, {'file', 'link'})

    def test_six_exclusions_never_turn_reason_metadata_into_host_calls_or_passes(self):
        with patch.object(c.safe, 'capture_command') as child:
            for row in self.fixture['vm_only']:
                self.assertIn('reason', row)
                with self.assertRaises(c.safe.OracleError): c.host_argv('/proc/self/fd/71', row, self.fixture['cases'])
            child.assert_not_called()
        self.assertEqual(len(c.output_paths(self.fixture)), len(set(c.output_paths(self.fixture))))
        self.assertTrue(all('/work/' in p or not p.endswith('-records') for p in c.output_paths(self.fixture)))

    def test_readonly_verifier_checks_full_synthetic_retained_capture_and_rejects_tampering(self):
        # Synthetic provenance only: no actual evaluator/GNU observation is claimed.
        with patch.object(c, 'CORE_PINS', {}), patch.object(c, 'PROBE_SOURCE_SHA', c.digest(b'SYNTHETIC source')):
            expected = self.expected()
            table = {p['path']: p for p in expected['pins']}
            out = self.root / c.BASE / c.CAPTURE; out.mkdir(parents=True)
            source = out / 'source'; source.mkdir()
            for i, name in enumerate(c.SOURCES):
                raw = self.raw if name == c.FIXTURE else b'SYNTHETIC source'
                (source / f'file{i}.raw').write_bytes(raw)
                table[str(self.root / name)].update(bytes=len(raw), sha256=c.digest(raw))
            (source / 'INDEX.json').write_bytes(c.canonical(list(c.SOURCES)))
            for name, original in [('probe.elf', str(self.root / c.PROBE)), ('grep.elf', str(c.shared.GREP))]:
                raw = ('SYNTHETIC ' + name).encode(); (out / name).write_bytes(raw)
                table[original].update(bytes=len(raw), sha256=c.digest(raw))
            expected['compiled_sources'] = {n: table[str(self.root / n)]['sha256'] for n in c.COMPILED}
            c.check_control(self.root, expected)
            approved = c.canonical({**expected, 'root_reviewed': True})
            approval = self.root / c.BASE / (c.CAPTURE + '-APPROVED.json'); approval.write_bytes(approved)
            (out / 'CONTROL.json').write_bytes(approved)
            probe = self.probe(expected)
            observations = {
                'probe-identity': {'stdout': c.canonical(c.identity(self.root, expected)).decode(), 'stderr': '', 'status': 0},
                'probe-outcomes': {'stdout': c.canonical(probe).decode(), 'stderr': '', 'status': 0},
                'grep-version': {'stdout': '71 (GNU grep) 3.12\n', 'stderr': '', 'status': 0}}
            comparisons = []
            for i, (row, vm) in enumerate(zip(self.fixture['cases'], probe['outcomes'])):
                result = {k: row['expected'][k] for k in ('stdout', 'stderr', 'status')}
                observations[f'case{i}'] = result
                comparison = c.compared(row, vm['outcome'], result, row['files'])
                comparisons.append(comparison)
                (out / f'row{i}.json').write_bytes(c.canonical(comparison))
            for name, observed in observations.items():
                folder = out / name; folder.mkdir(); (folder / 'work').mkdir()
                for key in ('stdout', 'stderr'): (folder / (key + '.raw')).write_bytes(observed[key].encode())
                (folder / 'RESULT.json').write_bytes(c.canonical(observed))
                (folder / 'STATUS.json').write_bytes(c.canonical({'complete': True, 'status': observed['status']}))
                if name.startswith('case'):
                    for filename, text in self.fixture['cases'][int(name[4:])]['files'].items():
                        (folder / 'work' / filename).write_bytes(text.encode())
            manifest = {'schema_version': 1, 'control': expected, 'GNU_descriptor': '/proc/self/fd/71', 'accepted': True,
                'claims': expected['scope'], 'GNU_exclusions_executed': False, 'limits': {},
                'comparisons': comparisons, 'GNU_exclusions': self.fixture['vm_only'],
                'outputs': [c.evidence_pin(out, path) for path in c.output_paths(self.fixture)]}
            def seal():
                raw = c.canonical(manifest); (out / 'MANIFEST.json').write_bytes(raw)
                for name in ['COMPLETE', 'COMPLETION-PREPARED']:
                    (out / name).write_bytes((c.digest(raw) + '\n').encode())
            seal()
            with patch.object(c.shared, 'root_guard', return_value=()), patch.object(c.safe, 'capture_command') as child:
                self.assertEqual(c.verify(self.root)['status'], 'VERIFIED24-FIXED-UTILITY-COMPARISONS')
                child.assert_not_called()
                original = (out / 'case0/stdout.raw').read_bytes()
                (out / 'case0/stdout.raw').write_bytes(b'forged')
                with self.assertRaises(c.safe.OracleError): c.verify(self.root)
                (out / 'case0/stdout.raw').write_bytes(original)
                approval.write_bytes(c.canonical({**expected, 'root_reviewed': False}))
                with self.assertRaises(c.safe.OracleError): c.verify(self.root)
                approval.write_bytes(approved)
                manifest['accepted'] = False; seal()
                with self.assertRaises(c.safe.OracleError): c.verify(self.root)
                child.assert_not_called()


    def test_real_library_probe_identity_has_the_complete_closed396_input_superset(self):
        import re
        source = (PROJECT / c.PROBE_SOURCE).read_text()
        self.assertIn('use dlms_learning_lab::shell;', source)
        self.assertNotIn('#[path = "../src/shell.rs"]', source)
        self.assertNotIn('#[path = "../src/ere.rs"]', source)
        table = source.split('const COMPILED_INPUTS:', 1)[1].split('fn identity()', 1)[0]
        pairs = re.findall(r'\(\s*"([^"]+)",\s*include_bytes!\(\s*"([^"]+)"\s*\)\s*,?\s*\)', table)
        self.assertEqual(tuple(name for name, _ in pairs), c.COMPILED)
        self.assertTrue(all(include == '../' + name for name, include in pairs))
        self.assertEqual(len(c.COMPILED), 396)
        self.assertEqual(len(c.COMPILED), len(set(c.COMPILED)))
        self.assertEqual(len(c.SOURCES), 409)
        self.assertEqual(len(c.SOURCES), len(set(c.SOURCES)))
        actual = {p.relative_to(PROJECT).as_posix() for family in c.SOURCE_FAMILIES
                  for p in (PROJECT / family).rglob('*') if p.is_file()}
        current_with_ospf_broadcast_foundation=actual
        ospf_broadcast_foundation_extension={name for name,_,_ in CURRENT_OSPF_BROADCAST_FOUNDATION_INPUTS}
        self.assertEqual(len(ospf_broadcast_foundation_extension),9)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_OSPF_BROADCAST_FOUNDATION_INPUTS']
        self.assertTrue(ospf_broadcast_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ospf_broadcast_foundation=actual-ospf_broadcast_foundation_extension
        actual=actual_before_ospf_broadcast_foundation
        current_with_route_reading_foundation=actual
        route_reading_foundation_extension={name for name,_,_ in CURRENT_ROUTE_READING_FOUNDATION_INPUTS}
        self.assertEqual(len(route_reading_foundation_extension),9)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_ROUTE_READING_FOUNDATION_INPUTS']
        self.assertTrue(route_reading_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_route_reading_foundation=actual-route_reading_foundation_extension
        actual=actual_before_route_reading_foundation
        current_with_nfs_export_foundation=actual
        nfs_export_foundation_extension={name for name,_,_ in CURRENT_NFS_EXPORT_FOUNDATION_INPUTS}
        self.assertEqual(len(nfs_export_foundation_extension),9)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_NFS_EXPORT_FOUNDATION_INPUTS']
        self.assertTrue(nfs_export_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_nfs_export_foundation=actual-nfs_export_foundation_extension
        actual=actual_before_nfs_export_foundation
        current_with_vlan_gateway_foundation=actual
        vlan_gateway_foundation_extension={name for name,_,_ in CURRENT_VLAN_GATEWAY_FOUNDATION_INPUTS}
        self.assertEqual(len(vlan_gateway_foundation_extension),9)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_VLAN_GATEWAY_FOUNDATION_INPUTS']
        self.assertTrue(vlan_gateway_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_vlan_gateway_foundation=actual-vlan_gateway_foundation_extension
        actual=actual_before_vlan_gateway_foundation
        current_with_host_dns_security_foundation=actual
        host_dns_security_foundation_extension={name for name,_,_ in CURRENT_HOST_DNS_SECURITY_FOUNDATION_INPUTS}
        self.assertEqual(len(host_dns_security_foundation_extension),12)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_HOST_DNS_SECURITY_FOUNDATION_INPUTS']
        self.assertTrue(host_dns_security_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_host_dns_security_foundation=actual-host_dns_security_foundation_extension
        actual=actual_before_host_dns_security_foundation
        current_with_site_connectivity_foundation=actual
        site_connectivity_foundation_extension={name for name,_,_ in CURRENT_SITE_CONNECTIVITY_FOUNDATION_INPUTS}
        self.assertEqual(len(site_connectivity_foundation_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SITE_CONNECTIVITY_FOUNDATION_INPUTS']
        self.assertTrue(site_connectivity_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_site_connectivity_foundation=actual-site_connectivity_foundation_extension
        actual=actual_before_site_connectivity_foundation
        current_with_network_evidence_review_foundation=actual
        network_evidence_review_foundation_extension={name for name,_,_ in CURRENT_NETWORK_EVIDENCE_REVIEW_FOUNDATION_INPUTS}
        self.assertEqual(len(network_evidence_review_foundation_extension),9)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_NETWORK_EVIDENCE_REVIEW_FOUNDATION_INPUTS']
        self.assertTrue(network_evidence_review_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_network_evidence_review_foundation=actual-network_evidence_review_foundation_extension
        actual=actual_before_network_evidence_review_foundation
        current_with_staged_network_foundation=actual
        staged_network_foundation_extension={name for name,_,_ in CURRENT_STAGED_NETWORK_FOUNDATION_INPUTS}
        self.assertEqual(len(staged_network_foundation_extension),14)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_STAGED_NETWORK_FOUNDATION_INPUTS']
        self.assertTrue(staged_network_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_staged_network_foundation=actual-staged_network_foundation_extension
        actual=actual_before_staged_network_foundation
        current_with_credentials_vpn_foundation=actual
        credentials_vpn_foundation_extension={name for name,_,_ in CURRENT_CREDENTIALS_VPN_FOUNDATION_INPUTS}
        self.assertEqual(len(credentials_vpn_foundation_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_CREDENTIALS_VPN_FOUNDATION_INPUTS']
        self.assertTrue(credentials_vpn_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_credentials_vpn_foundation=actual-credentials_vpn_foundation_extension
        actual=actual_before_credentials_vpn_foundation
        current_with_security_investigation_foundation=actual
        security_investigation_foundation_extension={name for name,_,_ in CURRENT_SECURITY_INVESTIGATION_FOUNDATION_INPUTS}
        self.assertEqual(len(security_investigation_foundation_extension),15)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SECURITY_INVESTIGATION_FOUNDATION_INPUTS']
        self.assertTrue(security_investigation_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_security_investigation_foundation=actual-security_investigation_foundation_extension
        actual=actual_before_security_investigation_foundation
        current_with_desktop_profile_foundation=actual
        desktop_profile_foundation_extension={name for name,_,_ in CURRENT_DESKTOP_PROFILE_FOUNDATION_INPUTS}
        self.assertEqual(len(desktop_profile_foundation_extension),15)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DESKTOP_PROFILE_FOUNDATION_INPUTS']
        self.assertTrue(desktop_profile_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_desktop_profile_foundation=actual-desktop_profile_foundation_extension
        actual=actual_before_desktop_profile_foundation
        current_with_apache_https_foundation=actual
        apache_https_foundation_extension={name for name,_,_ in CURRENT_APACHE_HTTPS_FOUNDATION_INPUTS}
        self.assertEqual(len(apache_https_foundation_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_APACHE_HTTPS_FOUNDATION_INPUTS']
        self.assertTrue(apache_https_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_apache_https_foundation=actual-apache_https_foundation_extension
        actual=actual_before_apache_https_foundation
        current_with_ata_smart_foundation=actual
        ata_smart_foundation_extension={name for name,_,_ in CURRENT_ATA_SMART_FOUNDATION_INPUTS}
        self.assertEqual(len(ata_smart_foundation_extension),14)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_ATA_SMART_FOUNDATION_INPUTS']
        self.assertTrue(ata_smart_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ata_smart_foundation=actual-ata_smart_foundation_extension
        actual=actual_before_ata_smart_foundation
        current_with_iscsi_foundation=actual
        iscsi_foundation_extension={name for name,_,_ in CURRENT_ISCSI_FOUNDATION_INPUTS}
        self.assertEqual(len(iscsi_foundation_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_ISCSI_FOUNDATION_INPUTS']
        self.assertTrue(iscsi_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_iscsi_foundation=actual-iscsi_foundation_extension
        actual=actual_before_iscsi_foundation
        current_with_pxe_foundation=actual
        pxe_foundation_extension={name for name,_,_ in CURRENT_PXE_FOUNDATION_INPUTS}
        self.assertEqual(len(pxe_foundation_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_PXE_FOUNDATION_INPUTS']
        self.assertTrue(pxe_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_pxe_foundation=actual-pxe_foundation_extension
        actual=actual_before_pxe_foundation
        current_with_initramfs_foundation=actual
        initramfs_foundation_extension={name for name,_,_ in CURRENT_INITRAMFS_FOUNDATION_INPUTS}
        self.assertEqual(len(initramfs_foundation_extension),12)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_INITRAMFS_FOUNDATION_INPUTS']
        self.assertTrue(initramfs_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_initramfs_foundation=actual-initramfs_foundation_extension
        actual=actual_before_initramfs_foundation
        current_with_dkms_foundation=actual
        dkms_foundation_extension={name for name,_,_ in CURRENT_DKMS_FOUNDATION_INPUTS}
        self.assertEqual(len(dkms_foundation_extension),12)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DKMS_FOUNDATION_INPUTS']
        self.assertTrue(dkms_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_dkms_foundation=actual-dkms_foundation_extension
        actual=actual_before_dkms_foundation
        current_with_kernel_build_foundation=actual
        kernel_build_foundation_extension={name for name,_,_ in CURRENT_KERNEL_BUILD_FOUNDATION_INPUTS}
        self.assertEqual(len(kernel_build_foundation_extension),14)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_KERNEL_BUILD_FOUNDATION_INPUTS']
        self.assertTrue(kernel_build_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_kernel_build_foundation=actual-kernel_build_foundation_extension
        actual=actual_before_kernel_build_foundation
        current_with_poe=actual
        poe_extension={name for name,_,_ in CURRENT_POE_INPUTS}
        self.assertEqual(len(poe_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_POE_INPUTS']
        self.assertTrue(poe_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_poe=actual-poe_extension
        actual=actual_before_poe
        current_with_campus_architecture=actual
        campus_architecture_extension={name for name,_,_ in CURRENT_CAMPUS_ARCHITECTURE_INPUTS}
        self.assertEqual(len(campus_architecture_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_CAMPUS_ARCHITECTURE_INPUTS']
        self.assertTrue(campus_architecture_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_campus_architecture=actual-campus_architecture_extension
        actual=actual_before_campus_architecture
        current_with_leaf_spine=actual
        leaf_spine_extension={name for name,_,_ in CURRENT_LEAF_SPINE_INPUTS}
        self.assertEqual(len(leaf_spine_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LEAF_SPINE_INPUTS']
        self.assertTrue(leaf_spine_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_leaf_spine=actual-leaf_spine_extension
        actual=actual_before_leaf_spine
        current_with_telnet_transport=actual
        telnet_transport_extension={name for name,_,_ in CURRENT_TELNET_TRANSPORT_INPUTS}
        self.assertEqual(len(telnet_transport_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_TELNET_TRANSPORT_INPUTS']
        self.assertTrue(telnet_transport_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_telnet_transport=actual-telnet_transport_extension
        actual=actual_before_telnet_transport
        current_with_nc_probe=actual
        nc_probe_extension={name for name,_,_ in CURRENT_NC_PROBE_INPUTS}
        self.assertEqual(len(nc_probe_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_NC_PROBE_INPUTS']
        self.assertTrue(nc_probe_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_nc_probe=actual-nc_probe_extension
        actual=actual_before_nc_probe
        current_with_ids_rule=actual
        ids_rule_extension={name for name,_,_ in CURRENT_IDS_RULE_INPUTS}
        self.assertEqual(len(ids_rule_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_IDS_RULE_INPUTS']
        self.assertTrue(ids_rule_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ids_rule=actual-ids_rule_extension
        actual=actual_before_ids_rule
        current_with_log_filter=actual
        log_filter_extension={name for name,_,_ in CURRENT_LOG_FILTER_INPUTS}
        self.assertEqual(len(log_filter_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LOG_FILTER_INPUTS']
        self.assertTrue(log_filter_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_log_filter=actual-log_filter_extension
        actual=actual_before_log_filter
        current_with_rpm_configuration=actual
        rpm_configuration_extension={name for name,_,_ in CURRENT_RPM_CONFIGURATION_INPUTS}
        self.assertEqual(len(rpm_configuration_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RPM_CONFIGURATION_INPUTS']
        self.assertTrue(rpm_configuration_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_rpm_configuration=actual-rpm_configuration_extension
        actual=actual_before_rpm_configuration
        current_with_vpn_point=actual
        vpn_point_extension={name for name,_,_ in CURRENT_VPN_POINT_INPUTS}
        self.assertEqual(len(vpn_point_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_VPN_POINT_INPUTS']
        self.assertTrue(vpn_point_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_vpn_point=actual-vpn_point_extension
        actual=actual_before_vpn_point
        current_with_service_exposure=actual
        service_exposure_extension={name for name,_,_ in CURRENT_SERVICE_EXPOSURE_INPUTS}
        self.assertEqual(len(service_exposure_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SERVICE_EXPOSURE_INPUTS']
        self.assertTrue(service_exposure_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_service_exposure=actual-service_exposure_extension
        actual=actual_before_service_exposure
        current_with_ftp_identity=actual
        ftp_identity_extension={name for name,_,_ in CURRENT_FTP_IDENTITY_INPUTS}
        self.assertEqual(len(ftp_identity_extension),14)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_FTP_IDENTITY_INPUTS']
        self.assertTrue(ftp_identity_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ftp_identity=actual-ftp_identity_extension
        actual=actual_before_ftp_identity
        current_with_linux_v6_startup=actual
        linux_v6_startup_extension={name for name,_,_ in CURRENT_LINUX_V6_STARTUP_INPUTS}
        self.assertEqual(len(linux_v6_startup_extension),12)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LINUX_V6_STARTUP_INPUTS']
        self.assertTrue(linux_v6_startup_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_linux_v6_startup=actual-linux_v6_startup_extension
        actual=actual_before_linux_v6_startup
        current_with_linux_v6_admin=actual
        linux_v6_admin_extension={name for name,_,_ in CURRENT_LINUX_V6_ADMIN_INPUTS}
        self.assertEqual(len(linux_v6_admin_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LINUX_V6_ADMIN_INPUTS']
        self.assertTrue(linux_v6_admin_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_linux_v6_admin=actual-linux_v6_admin_extension
        actual=actual_before_linux_v6_admin
        current_with_linux_service_rules=actual
        linux_service_rules_extension={name for name,_,_ in CURRENT_LINUX_SERVICE_RULE_INPUTS}
        self.assertEqual(len(linux_service_rules_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LINUX_SERVICE_RULE_INPUTS']
        self.assertTrue(linux_service_rules_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_linux_service_rules=actual-linux_service_rules_extension
        actual=actual_before_linux_service_rules
        current_with_linux_ipv6=actual
        linux_ipv6_extension={name for name,_,_ in CURRENT_LINUX_IPV6_INPUTS}
        self.assertEqual(len(linux_ipv6_extension),16)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LINUX_IPV6_INPUTS']
        self.assertTrue(linux_ipv6_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_linux_ipv6=actual-linux_ipv6_extension
        actual=actual_before_linux_ipv6
        current_with_linux_connections=actual
        linux_connections_extension={name for name,_,_ in CURRENT_LINUX_CONNECTION_INPUTS}
        self.assertEqual(len(linux_connections_extension),15)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LINUX_CONNECTION_INPUTS']
        self.assertTrue(linux_connections_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_linux_connections=actual-linux_connections_extension
        actual=actual_before_linux_connections
        current_with_linux_router=actual
        linux_router_extension={name for name,_,_ in CURRENT_LINUX_ROUTER_INPUTS}
        self.assertEqual(len(linux_router_extension),14)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LINUX_ROUTER_INPUTS']
        self.assertTrue(linux_router_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_linux_router=actual-linux_router_extension
        actual=actual_before_linux_router
        current_with_mailbox_admin=actual
        mailbox_admin_extension={name for name,_,_ in CURRENT_MAILBOX_ADMIN_INPUTS}
        self.assertEqual(len(mailbox_admin_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_MAILBOX_ADMIN_INPUTS']
        self.assertTrue(mailbox_admin_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_mailbox_admin=actual-mailbox_admin_extension
        actual=actual_before_mailbox_admin
        current_with_mailbox_access=actual
        mailbox_access_extension={name for name,_,_ in CURRENT_MAILBOX_ACCESS_INPUTS}
        self.assertEqual(len(mailbox_access_extension),15)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_MAILBOX_ACCESS_INPUTS']
        self.assertTrue(mailbox_access_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_mailbox_access=actual-mailbox_access_extension
        actual=actual_before_mailbox_access
        current_with_procmail_migration=actual
        procmail_migration_extension={name for name,_,_ in CURRENT_PROCMAIL_MIGRATION_INPUTS}
        self.assertEqual(len(procmail_migration_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_PROCMAIL_MIGRATION_INPUTS']
        self.assertTrue(procmail_migration_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_procmail_migration=actual-procmail_migration_extension
        actual=actual_before_procmail_migration
        current_with_sieve_vacation=actual
        sieve_vacation_extension={name for name,_,_ in CURRENT_SIEVE_VACATION_INPUTS}
        self.assertEqual(len(sieve_vacation_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SIEVE_VACATION_INPUTS']
        self.assertTrue(sieve_vacation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_sieve_vacation=actual-sieve_vacation_extension
        actual=actual_before_sieve_vacation
        current_with_sieve_delivery=actual
        sieve_delivery_extension={name for name,_,_ in CURRENT_SIEVE_DELIVERY_INPUTS}
        self.assertEqual(len(sieve_delivery_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SIEVE_DELIVERY_INPUTS']
        self.assertTrue(sieve_delivery_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_sieve_delivery=actual-sieve_delivery_extension
        actual=actual_before_sieve_delivery
        current_with_sieve_address=actual
        sieve_address_extension={name for name,_,_ in CURRENT_SIEVE_ADDRESS_INPUTS}
        self.assertEqual(len(sieve_address_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SIEVE_ADDRESS_INPUTS']
        self.assertTrue(sieve_address_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_sieve_address=actual-sieve_address_extension
        actual=actual_before_sieve_address
        current_with_sieve=actual
        sieve_extension={name for name,_,_ in CURRENT_SIEVE_INPUTS}
        self.assertEqual(len(sieve_extension),12)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SIEVE_INPUTS']
        self.assertTrue(sieve_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_sieve=actual-sieve_extension
        actual=actual_before_sieve
        current_with_smtp=actual
        smtp_extension={name for name,_,_ in CURRENT_SMTP_INPUTS}
        self.assertEqual(len(smtp_extension),14)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SMTP_INPUTS']
        self.assertTrue(smtp_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_smtp=actual-smtp_extension
        actual=actual_before_smtp
        current_with_guest_image=actual
        guest_image_extension={name for name,_,_ in CURRENT_GUEST_IMAGE_INPUTS}
        self.assertEqual(len(guest_image_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_GUEST_IMAGE_INPUTS']
        self.assertTrue(guest_image_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_guest_image=actual-guest_image_extension
        actual=actual_before_guest_image
        current_with_virtual_isolation=actual
        virtual_isolation_extension={name for name,_,_ in CURRENT_VIRTUAL_ISOLATION_INPUTS}
        self.assertEqual(len(virtual_isolation_extension),11)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_VIRTUAL_ISOLATION_INPUTS']
        self.assertTrue(virtual_isolation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_virtual_isolation=actual-virtual_isolation_extension
        actual=actual_before_virtual_isolation
        current_with_samba_share=actual
        samba_share_extension={name for name,_,_ in CURRENT_SAMBA_SHARE_INPUTS}
        self.assertEqual(len(samba_share_extension),12)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SAMBA_SHARE_INPUTS']
        self.assertTrue(samba_share_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_samba_share=actual-samba_share_extension
        actual=actual_before_samba_share
        current_with_network_transfer=actual
        network_transfer_extension={name for name,_,_ in CURRENT_NETWORK_TRANSFER_INPUTS}
        self.assertEqual(len(network_transfer_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_NETWORK_TRANSFER_INPUTS']
        self.assertTrue(network_transfer_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_network_transfer=actual-network_transfer_extension
        actual=actual_before_network_transfer
        current_with_handover_replay=actual
        handover_replay_extension={name for name,_,_ in CURRENT_HANDOVER_REPLAY_INPUTS}
        self.assertEqual(len(handover_replay_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_HANDOVER_REPLAY_INPUTS']
        self.assertTrue(handover_replay_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_handover_replay=actual-handover_replay_extension
        actual=actual_before_handover_replay
        current_with_editor_handover=actual
        editor_handover_extension={name for name,_,_ in CURRENT_EDITOR_HANDOVER_INPUTS}
        self.assertEqual(len(editor_handover_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_EDITOR_HANDOVER_INPUTS']
        self.assertTrue(editor_handover_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_editor_handover=actual-editor_handover_extension
        actual=actual_before_editor_handover
        current_with_modal_editing=actual
        modal_editing_extension={name for name,_,_ in CURRENT_MODAL_EDITING_INPUTS}
        self.assertEqual(len(modal_editing_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_MODAL_EDITING_INPUTS']
        self.assertTrue(modal_editing_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_modal_editing=actual-modal_editing_extension
        actual=actual_before_modal_editing
        current_with_ssh_access=actual
        ssh_access_extension={name for name,_,_ in CURRENT_SSH_ACCESS_INPUTS}
        self.assertEqual(len(ssh_access_extension),11)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SSH_ACCESS_INPUTS']
        self.assertTrue(ssh_access_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ssh_access=actual-ssh_access_extension
        actual=actual_before_ssh_access
        current_with_dns_query_tools=actual
        dns_query_tools_extension={name for name,_,_ in CURRENT_DNS_QUERY_TOOLS_INPUTS}
        self.assertEqual(len(dns_query_tools_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DNS_QUERY_TOOLS_INPUTS']
        self.assertTrue(dns_query_tools_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_dns_query_tools=actual-dns_query_tools_extension
        actual=actual_before_dns_query_tools
        current_with_direct_dns=actual
        direct_dns_extension={name for name,_,_ in CURRENT_DIRECT_DNS_INPUTS}
        self.assertEqual(len(direct_dns_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DIRECT_DNS_INPUTS']
        self.assertTrue(direct_dns_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_direct_dns=actual-direct_dns_extension
        actual=actual_before_direct_dns
        current_with_client_diagnostic=actual
        client_diagnostic_extension={name for name,_,_ in CURRENT_CLIENT_DIAGNOSTIC_INPUTS}
        self.assertEqual(len(client_diagnostic_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_CLIENT_DIAGNOSTIC_INPUTS']
        self.assertTrue(client_diagnostic_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_client_diagnostic=actual-client_diagnostic_extension
        actual=actual_before_client_diagnostic
        current_with_rest_controller=actual
        rest_controller_extension={name for name,_,_ in CURRENT_REST_CONTROLLER_INPUTS}
        self.assertEqual(len(rest_controller_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_REST_CONTROLLER_INPUTS']
        self.assertTrue(rest_controller_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_rest_controller=actual-rest_controller_extension
        actual=actual_before_rest_controller
        current_with_host_echo=actual
        host_echo_extension={name for name,_,_ in CURRENT_HOST_ECHO_INPUTS}
        self.assertEqual(len(host_echo_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_HOST_ECHO_INPUTS']
        self.assertTrue(host_echo_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_host_echo=actual-host_echo_extension
        actual=actual_before_host_echo
        current_with_protocol_evidence=actual
        protocol_evidence_extension={name for name,_,_ in CURRENT_PROTOCOL_EVIDENCE_INPUTS}
        self.assertEqual(len(protocol_evidence_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_PROTOCOL_EVIDENCE_INPUTS']
        self.assertTrue(protocol_evidence_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_protocol_evidence=actual-protocol_evidence_extension
        actual=actual_before_protocol_evidence
        current_with_mixed_ip=actual
        mixed_ip_extension={name for name,_,_ in CURRENT_MIXED_IP_INPUTS}
        self.assertEqual(len(mixed_ip_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_MIXED_IP_INPUTS']
        self.assertTrue(mixed_ip_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_mixed_ip=actual-mixed_ip_extension
        actual=actual_before_mixed_ip
        current_with_service_names=actual
        service_names_extension={name for name,_,_ in CURRENT_SERVICE_NAMES_INPUTS}
        self.assertEqual(len(service_names_extension),13)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SERVICE_NAMES_INPUTS']
        self.assertTrue(service_names_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_service_names=actual-service_names_extension
        actual=actual_before_service_names
        current_with_reverse_proxy=actual
        reverse_proxy_extension={name for name,_,_ in CURRENT_REVERSE_PROXY_INPUTS}
        self.assertEqual(len(reverse_proxy_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_REVERSE_PROXY_INPUTS']
        self.assertTrue(reverse_proxy_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_reverse_proxy=actual-reverse_proxy_extension
        actual=actual_before_reverse_proxy
        current_with_http_cache=actual
        http_cache_extension={name for name,_,_ in CURRENT_HTTP_CACHE_INPUTS}
        self.assertEqual(len(http_cache_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_HTTP_CACHE_INPUTS']
        self.assertTrue(http_cache_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_http_cache=actual-http_cache_extension
        actual=actual_before_http_cache
        current_with_proxy_policy=actual
        proxy_policy_extension={name for name,_,_ in CURRENT_PROXY_POLICY_INPUTS}
        self.assertEqual(len(proxy_policy_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_PROXY_POLICY_INPUTS']
        self.assertTrue(proxy_policy_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_proxy_policy=actual-proxy_policy_extension
        actual=actual_before_proxy_policy
        current_with_snmp=actual
        snmp_extension={name for name,_,_ in CURRENT_SNMP_INPUTS}
        self.assertEqual(len(snmp_extension),12)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SNMP_INPUTS']
        self.assertTrue(snmp_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_snmp=actual-snmp_extension
        actual=actual_before_snmp
        current_with_wlan_paths=actual
        wlan_paths_extension={name for name,_,_ in CURRENT_WLAN_PATH_INPUTS}
        self.assertEqual(len(wlan_paths_extension),11)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_WLAN_PATH_INPUTS']
        self.assertTrue(wlan_paths_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_wlan_paths=actual-wlan_paths_extension
        actual=actual_before_wlan_paths
        current_with_wlan_identity=actual
        wlan_identity_extension={name for name,_,_ in CURRENT_WLAN_IDENTITY_INPUTS}
        self.assertEqual(len(wlan_identity_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_WLAN_IDENTITY_INPUTS']
        self.assertTrue(wlan_identity_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_wlan_identity=actual-wlan_identity_extension
        actual=actual_before_wlan_identity
        current_with_wireless_channels=actual
        wireless_channels_extension={name for name,_,_ in CURRENT_WIRELESS_CHANNEL_INPUTS}
        self.assertEqual(len(wireless_channels_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_WIRELESS_CHANNEL_INPUTS']
        self.assertTrue(wireless_channels_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_wireless_channels=actual-wireless_channels_extension
        actual=actual_before_wireless_channels
        current_with_timezones=actual
        timezones_extension={name for name,_,_ in CURRENT_TIMEZONES_INPUTS}
        self.assertEqual(len(timezones_extension),14)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_TIMEZONES_INPUTS']
        self.assertTrue(timezones_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_timezones=actual-timezones_extension
        actual=actual_before_timezones
        current_with_localization=actual
        localization_extension={name for name,_,_ in CURRENT_LOCALIZATION_INPUTS}
        self.assertEqual(len(localization_extension),16)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LOCALIZATION_INPUTS']
        self.assertTrue(localization_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_localization=actual-localization_extension
        actual=actual_before_localization
        current_with_print_queues=actual
        print_queues_extension={name for name,_,_ in CURRENT_PRINT_QUEUES_INPUTS}
        self.assertEqual(len(print_queues_extension),14)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_PRINT_QUEUES_INPUTS']
        self.assertTrue(print_queues_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_print_queues=actual-print_queues_extension
        actual=actual_before_print_queues
        current_with_dns_access=actual
        dns_access_extension={name for name,_,_ in CURRENT_DNS_ACCESS_INPUTS}
        self.assertEqual(len(dns_access_extension),11)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DNS_ACCESS_INPUTS']
        self.assertTrue(dns_access_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_dns_access=actual-dns_access_extension
        actual=actual_before_dns_access
        current_with_source_install=actual
        source_install_extension={name for name,_,_ in CURRENT_SOURCE_INSTALL_INPUTS}
        self.assertEqual(len(source_install_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SOURCE_INSTALL_INPUTS']
        self.assertTrue(source_install_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_source_install=actual-source_install_extension
        actual=actual_before_source_install
        current_with_maintenance_notice=actual
        maintenance_notice_extension={name for name,_,_ in CURRENT_MAINTENANCE_NOTICE_INPUTS}
        self.assertEqual(len(maintenance_notice_extension),9)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_MAINTENANCE_NOTICE_INPUTS']
        self.assertTrue(maintenance_notice_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_maintenance_notice=actual-maintenance_notice_extension
        actual=actual_before_maintenance_notice
        current_with_capacity_forecast=actual
        capacity_forecast_extension={name for name,_,_ in CURRENT_CAPACITY_FORECAST_INPUTS}
        self.assertEqual(len(capacity_forecast_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_CAPACITY_FORECAST_INPUTS']
        self.assertTrue(capacity_forecast_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_capacity_forecast=actual-capacity_forecast_extension
        actual=actual_before_capacity_forecast
        current_with_resource_observation=actual
        resource_observation_extension={name for name,_,_ in CURRENT_RESOURCE_OBSERVATION_INPUTS}
        self.assertEqual(len(resource_observation_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RESOURCE_OBSERVATION_INPUTS']
        self.assertTrue(resource_observation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_resource_observation=actual-resource_observation_extension
        actual=actual_before_resource_observation
        current_with_resource_measurement=actual
        resource_measurement_extension={name for name,_,_ in CURRENT_RESOURCE_MEASUREMENT_INPUTS}
        self.assertEqual(len(resource_measurement_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RESOURCE_MEASUREMENT_INPUTS']
        self.assertTrue(resource_measurement_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_resource_measurement=actual-resource_measurement_extension
        actual=actual_before_resource_measurement
        current_with_layout_policy=actual
        layout_policy_extension={name for name,_,_ in CURRENT_LAYOUT_POLICY_INPUTS}
        self.assertEqual(len(layout_policy_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LAYOUT_POLICY_INPUTS']
        self.assertTrue(layout_policy_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_layout_policy=actual-layout_policy_extension
        actual=actual_before_layout_policy
        current_with_json_inventory=actual
        json_inventory_extension={name for name,_,_ in CURRENT_JSON_INVENTORY_INPUTS}
        self.assertEqual(len(json_inventory_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_JSON_INVENTORY_INPUTS']
        self.assertTrue(json_inventory_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_json_inventory=actual-json_inventory_extension
        actual=actual_before_json_inventory
        current_with_wired_device=actual
        wired_device_extension={name for name,_,_ in CURRENT_WIRED_DEVICE_INPUTS}
        self.assertEqual(len(wired_device_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_WIRED_DEVICE_INPUTS']
        self.assertTrue(wired_device_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_wired_device=actual-wired_device_extension
        actual=actual_before_wired_device
        current_with_local_mail=actual
        local_mail_extension={name for name,_,_ in CURRENT_LOCAL_MAIL_INPUTS}
        self.assertEqual(len(local_mail_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LOCAL_MAIL_INPUTS']
        self.assertTrue(local_mail_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_local_mail=actual-local_mail_extension
        actual=actual_before_local_mail
        current_with_copied_journal=actual
        copied_journal_extension={name for name,_,_ in CURRENT_COPIED_JOURNAL_INPUTS}
        self.assertEqual(len(copied_journal_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_COPIED_JOURNAL_INPUTS']
        self.assertTrue(copied_journal_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_copied_journal=actual-copied_journal_extension
        actual=actual_before_copied_journal
        current_with_journal_archive=actual
        journal_archive_extension={name for name,_,_ in CURRENT_COPIED_JOURNAL_FOUNDATION_INPUTS}
        self.assertEqual(len(journal_archive_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_COPIED_JOURNAL_FOUNDATION_INPUTS']
        self.assertTrue(journal_archive_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_journal_archive=actual-journal_archive_extension
        actual=actual_before_journal_archive
        current_with_chrony_source=actual
        chrony_source_extension={name for name,_,_ in CURRENT_CHRONY_SOURCE_INPUTS}
        self.assertEqual(len(chrony_source_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_CHRONY_SOURCE_INPUTS']
        self.assertTrue(chrony_source_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_chrony_source=actual-chrony_source_extension
        actual=actual_before_chrony_source
        current_with_rescue_startup=actual
        rescue_startup_extension={name for name,_,_ in CURRENT_RESCUE_STARTUP_INPUTS}
        self.assertEqual(len(rescue_startup_extension),3)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RESCUE_STARTUP_INPUTS']
        self.assertTrue(rescue_startup_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_rescue_startup=actual-rescue_startup_extension
        actual=actual_before_rescue_startup
        current_with_grub_once=actual
        grub_once_extension={name for name,_,_ in CURRENT_GRUB_ONCE_INPUTS}
        self.assertEqual(len(grub_once_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_GRUB_ONCE_INPUTS']
        self.assertTrue(grub_once_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_grub_once=actual-grub_once_extension
        actual=actual_before_grub_once
        current_with_efi_removable=actual
        efi_removable_extension={name for name,_,_ in CURRENT_EFI_REMOVABLE_INPUTS}
        self.assertEqual(len(efi_removable_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_EFI_REMOVABLE_INPUTS']
        self.assertTrue(efi_removable_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_efi_removable=actual-efi_removable_extension
        actual=actual_before_efi_removable
        current_with_efi_loader=actual
        efi_loader_extension={name for name,_,_ in CURRENT_EFI_LOADER_INPUTS}
        self.assertEqual(len(efi_loader_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_EFI_LOADER_INPUTS']
        self.assertTrue(efi_loader_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_efi_loader=actual-efi_loader_extension
        actual=actual_before_efi_loader
        current_with_bios_loader=actual
        bios_loader_extension={name for name,_,_ in CURRENT_BIOS_LOADER_INPUTS}
        self.assertEqual(len(bios_loader_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_BIOS_LOADER_INPUTS']
        self.assertTrue(bios_loader_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_bios_loader=actual-bios_loader_extension
        actual=actual_before_bios_loader
        current_with_grub_settings=actual
        grub_settings_extension={name for name,_,_ in CURRENT_GRUB_SETTINGS_INPUTS}
        self.assertEqual(len(grub_settings_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_GRUB_SETTINGS_INPUTS']
        self.assertTrue(grub_settings_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_grub_settings=actual-grub_settings_extension
        actual=actual_before_grub_settings
        current_with_grub_menu=actual
        grub_menu_extension={name for name,_,_ in CURRENT_GRUB_MENU_INPUTS}
        self.assertEqual(len(grub_menu_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_GRUB_MENU_INPUTS']
        self.assertTrue(grub_menu_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_grub_menu=actual-grub_menu_extension
        actual=actual_before_grub_menu
        current_with_boot_entry=actual
        boot_entry_extension={name for name,_,_ in CURRENT_BOOT_ENTRY_INPUTS}
        self.assertEqual(len(boot_entry_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_BOOT_ENTRY_INPUTS']
        self.assertTrue(boot_entry_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_boot_entry=actual-boot_entry_extension
        actual=actual_before_boot_entry
        current_with_eui64=actual
        eui64_extension={name for name,_,_ in CURRENT_EUI64_INPUTS}
        self.assertEqual(len(eui64_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_EUI64_INPUTS']
        self.assertTrue(eui64_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_eui64=actual-eui64_extension
        actual=actual_before_eui64
        current_with_ldap_schema=actual
        ldap_schema_extension={name for name,_,_ in CURRENT_LDAP_SCHEMA_INPUTS}
        self.assertEqual(len(ldap_schema_extension),19)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LDAP_SCHEMA_INPUTS']
        self.assertTrue(ldap_schema_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ldap_schema=actual-ldap_schema_extension
        actual=actual_before_ldap_schema
        current_with_ldap_configuration=actual
        ldap_configuration_extension={name for name,_,_ in CURRENT_LDAP_CONFIGURATION_INPUTS}
        self.assertEqual(len(ldap_configuration_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LDAP_CONFIGURATION_INPUTS']
        self.assertTrue(ldap_configuration_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ldap_configuration=actual-ldap_configuration_extension
        actual=actual_before_ldap_configuration
        current_with_ldap_access=actual
        ldap_access_extension={name for name,_,_ in CURRENT_LDAP_ACCESS_INPUTS}
        self.assertEqual(len(ldap_access_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LDAP_ACCESS_INPUTS']
        self.assertTrue(ldap_access_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ldap_access=actual-ldap_access_extension
        actual=actual_before_ldap_access
        current_with_ldap_protection=actual
        ldap_protection_extension={name for name,_,_ in CURRENT_LDAP_PROTECTION_INPUTS}
        self.assertEqual(len(ldap_protection_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LDAP_PROTECTION_INPUTS']
        self.assertTrue(ldap_protection_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ldap_protection=actual-ldap_protection_extension
        actual=actual_before_ldap_protection
        current_with_ldap_delete=actual
        ldap_delete_extension={name for name,_,_ in CURRENT_LDAP_DELETE_INPUTS}
        self.assertEqual(len(ldap_delete_extension),3)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LDAP_DELETE_INPUTS']
        self.assertTrue(ldap_delete_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ldap_delete=actual-ldap_delete_extension
        actual=actual_before_ldap_delete
        current_with_ldap_utility=actual
        ldap_utility_extension={name for name,_,_ in CURRENT_LDAP_UTILITY_INPUTS}
        self.assertEqual(len(ldap_utility_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LDAP_UTILITY_INPUTS']
        self.assertTrue(ldap_utility_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ldap_utility=actual-ldap_utility_extension
        actual=actual_before_ldap_utility
        current_with_ldap_changes=actual
        ldap_changes_extension={name for name,_,_ in CURRENT_LDAP_CHANGE_INPUTS}
        self.assertEqual(len(ldap_changes_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LDAP_CHANGE_INPUTS']
        self.assertTrue(ldap_changes_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ldap_changes=actual-ldap_changes_extension
        actual=actual_before_ldap_changes
        current_with_ldap_query=actual
        ldap_query_extension={name for name,_,_ in CURRENT_LDAP_QUERY_INPUTS}
        self.assertEqual(len(ldap_query_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LDAP_QUERY_INPUTS']
        self.assertTrue(ldap_query_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_ldap_query=actual-ldap_query_extension
        actual=actual_before_ldap_query
        current_with_aaa_methods=actual
        aaa_methods_extension={name for name,_,_ in CURRENT_AAA_METHODS_INPUTS}
        self.assertEqual(len(aaa_methods_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_AAA_METHODS_INPUTS']
        self.assertTrue(aaa_methods_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_aaa_methods=actual-aaa_methods_extension
        actual=actual_before_aaa_methods
        current_with_local_access=actual
        local_access_extension={name for name,_,_ in CURRENT_LOCAL_ACCESS_INPUTS}
        self.assertEqual(len(local_access_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LOCAL_ACCESS_INPUTS']
        self.assertTrue(local_access_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_local_access=actual-local_access_extension
        actual=actual_before_local_access
        current_with_management_policy=actual
        management_policy_extension={name for name,_,_ in CURRENT_MANAGEMENT_POLICY_INPUTS}
        self.assertEqual(len(management_policy_extension),9)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_MANAGEMENT_POLICY_INPUTS']
        self.assertTrue(management_policy_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_management_policy=actual-management_policy_extension
        actual=actual_before_management_policy
        current_with_arp_inspection=actual
        arp_inspection_extension={name for name,_,_ in CURRENT_ARP_INSPECTION_INPUTS}
        self.assertEqual(len(arp_inspection_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_ARP_INSPECTION_INPUTS']
        self.assertTrue(arp_inspection_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_arp_inspection=actual-arp_inspection_extension
        actual=actual_before_arp_inspection
        current_with_dhcp_snooping=actual
        dhcp_snooping_extension={name for name,_,_ in CURRENT_DHCP_SNOOPING_INPUTS}
        self.assertEqual(len(dhcp_snooping_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DHCP_SNOOPING_INPUTS']
        self.assertTrue(dhcp_snooping_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_dhcp_snooping=actual-dhcp_snooping_extension
        actual=actual_before_dhcp_snooping
        current_with_access_outputs=actual
        access_outputs_extension={name for name,_,_ in CURRENT_ACCESS_OUTPUTS_INPUTS}
        self.assertEqual(len(access_outputs_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_ACCESS_OUTPUTS_INPUTS']
        self.assertTrue(access_outputs_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_access_outputs=actual-access_outputs_extension
        actual=actual_before_access_outputs
        current_with_port_security=actual
        port_security_extension={name for name,_,_ in CURRENT_PORT_SECURITY_INPUTS}
        self.assertEqual(len(port_security_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_PORT_SECURITY_INPUTS']
        self.assertTrue(port_security_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_port_security=actual-port_security_extension
        actual=actual_before_port_security
        current_with_static_cam=actual
        static_cam_extension={name for name,_,_ in CURRENT_STATIC_CAM_INPUTS}
        self.assertEqual(len(static_cam_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_STATIC_CAM_INPUTS']
        self.assertTrue(static_cam_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_static_cam=actual-static_cam_extension
        actual=actual_before_static_cam
        current_with_aging=actual
        aging_extension={name for name,_,_ in CURRENT_AGING_INPUTS}
        self.assertEqual(len(aging_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_AGING_INPUTS']
        self.assertTrue(aging_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_aging=actual-aging_extension
        actual=actual_before_aging
        current_with_pam_control=actual
        pam_control_extension={name for name,_,_ in CURRENT_PAM_CONTROL_INPUTS}
        self.assertEqual(len(pam_control_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_PAM_CONTROL_INPUTS']
        self.assertTrue(pam_control_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_pam_control=actual-pam_control_extension
        actual=actual_before_pam_control
        current_with_bridge_foundation=actual
        bridge_foundation_extension={name for name,_,_ in CURRENT_BRIDGE_FOUNDATION_INPUTS}
        self.assertEqual(len(bridge_foundation_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_BRIDGE_FOUNDATION_INPUTS']
        self.assertTrue(bridge_foundation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_bridge_foundation=actual-bridge_foundation_extension
        actual=actual_before_bridge_foundation
        current_with_bond_control=actual
        bond_control_extension={name for name,_,_ in CURRENT_BOND_CONTROL_INPUTS}
        self.assertEqual(len(bond_control_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_BOND_CONTROL_INPUTS']
        self.assertTrue(bond_control_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_bond_control=actual-bond_control_extension
        actual=actual_before_bond_control
        current_with_vlan_host=actual
        vlan_host_extension={name for name,_,_ in CURRENT_VLAN_HOST_INPUTS}
        self.assertEqual(len(vlan_host_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_VLAN_HOST_INPUTS']
        self.assertTrue(vlan_host_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_vlan_host=actual-vlan_host_extension
        actual=actual_before_vlan_host
        current_with_raid_assembly=actual
        raid_assembly_extension={name for name,_,_ in CURRENT_RAID_ASSEMBLY_INPUTS}
        self.assertEqual(len(raid_assembly_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RAID_ASSEMBLY_INPUTS']
        self.assertTrue(raid_assembly_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_raid_assembly=actual-raid_assembly_extension
        actual=actual_before_raid_assembly
        current_with_autofs=actual
        autofs_extension={name for name,_,_ in CURRENT_AUTOFS_INPUTS}
        self.assertEqual(len(autofs_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_AUTOFS_INPUTS']
        self.assertTrue(autofs_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_autofs=actual-autofs_extension
        actual=actual_before_autofs
        current_with_lvm_mount=actual
        lvm_mount_extension={name for name,_,_ in CURRENT_LVM_MOUNT_INPUTS}
        self.assertEqual(len(lvm_mount_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LVM_MOUNT_INPUTS']
        self.assertTrue(lvm_mount_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_lvm_mount=actual-lvm_mount_extension
        actual=actual_before_lvm_mount
        current_with_lvm_config_scan=actual
        lvm_config_scan_extension={name for name,_,_ in CURRENT_LVM_CONFIG_SCAN_INPUTS}
        self.assertEqual(len(lvm_config_scan_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LVM_CONFIG_SCAN_INPUTS']
        self.assertTrue(lvm_config_scan_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_lvm_config_scan=actual-lvm_config_scan_extension
        actual=actual_before_lvm_config_scan
        current_with_device_path_model=actual
        device_path_model_extension={name for name,_,_ in CURRENT_DEVICE_PATH_MODEL_INPUTS}
        self.assertEqual(len(device_path_model_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEVICE_PATH_MODEL_INPUTS']
        self.assertTrue(device_path_model_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_device_path_model=actual-device_path_model_extension
        actual=actual_before_device_path_model
        current_with_cow_model=actual
        cow_model_extension={name for name,_,_ in CURRENT_COW_MODEL_INPUTS}
        self.assertEqual(len(cow_model_extension),9)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_COW_MODEL_INPUTS']
        self.assertTrue(cow_model_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_cow_model=actual-cow_model_extension
        actual=actual_before_cow_model
        current_with_mbr_layout=actual
        mbr_layout_extension={name for name,_,_ in CURRENT_MBR_LAYOUT_INPUTS}
        self.assertEqual(len(mbr_layout_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_MBR_LAYOUT_INPUTS']
        self.assertTrue(mbr_layout_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_mbr_layout=actual-mbr_layout_extension
        actual=actual_before_mbr_layout
        current_with_pv_capacity=actual
        pv_capacity_extension={name for name,_,_ in CURRENT_PV_CAPACITY_INPUTS}
        self.assertEqual(len(pv_capacity_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_PV_CAPACITY_INPUTS']
        self.assertTrue(pv_capacity_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_pv_capacity=actual-pv_capacity_extension
        actual=actual_before_pv_capacity
        current_with_lvm_rename=actual
        lvm_rename_extension={name for name,_,_ in CURRENT_LVM_RENAME_INPUTS}
        self.assertEqual(len(lvm_rename_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LVM_RENAME_INPUTS']
        self.assertTrue(lvm_rename_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_lvm_rename=actual-lvm_rename_extension
        actual=actual_before_lvm_rename
        current_with_lvm_activation=actual
        lvm_activation_extension={name for name,_,_ in CURRENT_LVM_ACTIVATION_INPUTS}
        self.assertEqual(len(lvm_activation_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LVM_ACTIVATION_INPUTS']
        self.assertTrue(lvm_activation_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_lvm_activation=actual-lvm_activation_extension
        actual=actual_before_lvm_activation
        current_with_lvm_retirement=actual
        lvm_retirement_extension={name for name,_,_ in CURRENT_LVM_RETIREMENT_INPUTS}
        self.assertEqual(len(lvm_retirement_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LVM_RETIREMENT_INPUTS']
        self.assertTrue(lvm_retirement_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_lvm_retirement=actual-lvm_retirement_extension
        actual=actual_before_lvm_retirement
        current_with_lvm_provisioning=actual
        lvm_provisioning_extension={name for name,_,_ in CURRENT_LVM_PROVISIONING_INPUTS}
        self.assertEqual(len(lvm_provisioning_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LVM_PROVISIONING_INPUTS']
        self.assertTrue(lvm_provisioning_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_lvm_provisioning=actual-lvm_provisioning_extension
        actual=actual_before_lvm_provisioning
        current_with_swap_policy=actual
        swap_policy_extension={name for name,_,_ in CURRENT_SWAP_POLICY_INPUTS}
        self.assertEqual(len(swap_policy_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SWAP_POLICY_INPUTS']
        self.assertTrue(swap_policy_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_swap_policy=actual-swap_policy_extension
        actual=actual_before_swap_policy
        current_with_host_ipv6=actual
        host_ipv6_extension={name for name,_,_ in CURRENT_HOST_IPV6_INPUTS}
        self.assertEqual(len(host_ipv6_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_HOST_IPV6_INPUTS']
        self.assertTrue(host_ipv6_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_host_ipv6=actual-host_ipv6_extension
        actual=actual_before_host_ipv6
        current_with_rpm_metadata=actual
        rpm_metadata_extension={name for name,_,_ in CURRENT_RPM_METADATA_INPUTS}
        self.assertEqual(len(rpm_metadata_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RPM_METADATA_INPUTS']
        self.assertTrue(rpm_metadata_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_rpm_metadata=actual-rpm_metadata_extension
        actual=actual_before_rpm_metadata
        current_with_log_triage=actual
        log_triage_extension={name for name,_,_ in CURRENT_LOG_TRIAGE_INPUTS}
        self.assertEqual(len(log_triage_extension),3)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LOG_TRIAGE_INPUTS']
        self.assertTrue(log_triage_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_log_triage=actual-log_triage_extension
        actual=actual_before_log_triage
        current_with_kernel_names=actual
        kernel_names_extension={name for name,_,_ in CURRENT_KERNEL_NAMES_INPUTS}
        self.assertEqual(len(kernel_names_extension),3)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_KERNEL_NAMES_INPUTS']
        self.assertTrue(kernel_names_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_kernel_names=actual-kernel_names_extension
        actual=actual_before_kernel_names
        current_with_kernel_module=actual
        kernel_module_extension={name for name,_,_ in CURRENT_KERNEL_MODULE_INPUTS}
        self.assertEqual(len(kernel_module_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_KERNEL_MODULE_INPUTS']
        self.assertTrue(kernel_module_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_kernel_module=actual-kernel_module_extension
        actual=actual_before_kernel_module
        current_with_size_search=actual
        size_search_extension={name for name,_,_ in CURRENT_SIZE_SEARCH_INPUTS}
        self.assertEqual(len(size_search_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SIZE_SEARCH_INPUTS']
        self.assertTrue(size_search_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_size_search=actual-size_search_extension
        actual=actual_before_size_search
        current_with_recursive_tree=actual
        recursive_tree_extension={name for name,_,_ in CURRENT_RECURSIVE_TREE_INPUTS}
        self.assertEqual(len(recursive_tree_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RECURSIVE_TREE_INPUTS']
        self.assertTrue(recursive_tree_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_recursive_tree=actual-recursive_tree_extension
        actual=actual_before_recursive_tree
        current_with_transport=actual
        transport_extension={name for name,_,_ in CURRENT_TRANSPORT_INPUTS}
        self.assertEqual(len(transport_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_TRANSPORT_INPUTS']
        self.assertTrue(transport_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_transport=actual-transport_extension
        actual=actual_before_transport
        current_with_content_backup_lesson=actual
        content_backup_lesson_extension={name for name,_,_ in CURRENT_CONTENT_BACKUP_LESSON_INPUTS}
        self.assertEqual(len(content_backup_lesson_extension),3)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_CONTENT_BACKUP_LESSON_INPUTS']
        self.assertTrue(content_backup_lesson_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_content_backup_lesson=actual-content_backup_lesson_extension
        actual=actual_before_content_backup_lesson
        current_with_directory_policy_workspace=actual
        directory_policy_workspace_extension={name for name,_,_ in CURRENT_DIRECTORY_POLICY_WORKSPACE_INPUTS}
        self.assertEqual(len(directory_policy_workspace_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DIRECTORY_POLICY_WORKSPACE_INPUTS']
        self.assertTrue(directory_policy_workspace_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_directory_policy_workspace=actual-directory_policy_workspace_extension
        actual=actual_before_directory_policy_workspace
        current_with_directory_policy_core=actual
        directory_policy_core_extension={name for name,_,_ in CURRENT_DIRECTORY_POLICY_CORE_INPUTS}
        self.assertEqual(len(directory_policy_core_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DIRECTORY_POLICY_CORE_INPUTS']
        self.assertTrue(directory_policy_core_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_directory_policy_core=actual-directory_policy_core_extension
        actual=actual_before_directory_policy_core
        current_with_whereis_adapter=actual
        whereis_adapter_extension={name for name,_,_ in CURRENT_WHEREIS_ADAPTER_INPUTS}
        self.assertEqual(len(whereis_adapter_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_WHEREIS_ADAPTER_INPUTS']
        self.assertTrue(whereis_adapter_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_whereis_adapter=actual-whereis_adapter_extension
        actual=actual_before_whereis_adapter
        current_with_whereis_core=actual
        whereis_core_extension={name for name,_,_ in CURRENT_WHEREIS_CORE_INPUTS}
        self.assertEqual(len(whereis_core_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_WHEREIS_CORE_INPUTS']
        self.assertTrue(whereis_core_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_whereis_core=actual-whereis_core_extension
        actual=actual_before_whereis_core
        current_with_file_index_workspace=actual
        file_index_workspace_extension={name for name,_,_ in CURRENT_FILE_INDEX_WORKSPACE_INPUTS}
        self.assertEqual(len(file_index_workspace_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_FILE_INDEX_WORKSPACE_INPUTS']
        self.assertTrue(file_index_workspace_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_file_index_workspace=actual-file_index_workspace_extension
        actual=actual_before_file_index_workspace
        current_with_file_index_core=actual
        file_index_core_extension={name for name,_,_ in CURRENT_FILE_INDEX_CORE_INPUTS}
        self.assertEqual(len(file_index_core_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_FILE_INDEX_CORE_INPUTS']
        self.assertTrue(file_index_core_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_file_index_core=actual-file_index_core_extension
        actual=actual_before_file_index_core
        current_with_command_location_lesson=actual
        command_location_lesson_extension={name for name,_,_ in CURRENT_COMMAND_LOCATION_LESSON_INPUTS}
        self.assertEqual(len(command_location_lesson_extension),3)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_COMMAND_LOCATION_LESSON_INPUTS']
        self.assertTrue(command_location_lesson_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_command_location_lesson=actual-command_location_lesson_extension
        actual=actual_before_command_location_lesson
        current_with_command_location_binding=actual
        command_location_binding_extension={name for name,_,_ in CURRENT_COMMAND_LOCATION_BINDING_INPUTS}
        self.assertEqual(len(command_location_binding_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_COMMAND_LOCATION_BINDING_INPUTS']
        self.assertTrue(command_location_binding_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_command_location_binding=actual-command_location_binding_extension
        actual=actual_before_command_location_binding
        current_with_command_location_core=actual
        command_location_core_extension={name for name,_,_ in CURRENT_COMMAND_LOCATION_CORE_INPUTS}
        self.assertEqual(len(command_location_core_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_COMMAND_LOCATION_CORE_INPUTS']
        self.assertTrue(command_location_core_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_command_location_core=actual-command_location_core_extension
        actual=actual_before_command_location_core
        current_with_skeleton_adapter=actual
        skeleton_adapter_extension={name for name,_,_ in CURRENT_SKELETON_ADAPTER_INPUTS}
        self.assertEqual(len(skeleton_adapter_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SKELETON_ADAPTER_INPUTS']
        self.assertTrue(skeleton_adapter_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_skeleton_adapter=actual-skeleton_adapter_extension
        actual=actual_before_skeleton_adapter
        current_with_skeleton_core=actual
        skeleton_core_extension={name for name,_,_ in CURRENT_SKELETON_CORE_INPUTS}
        self.assertEqual(len(skeleton_core_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SKELETON_CORE_INPUTS']
        self.assertTrue(skeleton_core_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_skeleton_core=actual-skeleton_core_extension
        actual=actual_before_skeleton_core
        current_with_path_lookup=actual
        path_lookup_extension={name for name,_,_ in CURRENT_PATH_LOOKUP_INPUTS}
        self.assertEqual(len(path_lookup_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_PATH_LOOKUP_INPUTS']
        self.assertTrue(path_lookup_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_path_lookup=actual-path_lookup_extension
        actual=actual_before_path_lookup
        current_with_listener=actual
        listener_extension={name for name,_,_ in CURRENT_LISTENER_INPUTS}
        self.assertEqual(len(listener_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LISTENER_INPUTS']
        self.assertTrue(listener_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_listener=actual-listener_extension
        actual=actual_before_listener
        current_with_managed_config=actual
        managed_config_extension={name for name,_,_ in CURRENT_MANAGED_CONFIG_INPUTS}
        self.assertEqual(len(managed_config_extension),3)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_MANAGED_CONFIG_INPUTS']
        self.assertTrue(managed_config_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_managed_config=actual-managed_config_extension
        actual=actual_before_managed_config
        current_with_fhs_content=actual
        fhs_content_extension={name for name,_,_ in CURRENT_FHS_CONTENT_INPUTS}
        self.assertEqual(len(fhs_content_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_FHS_CONTENT_INPUTS']
        self.assertTrue(fhs_content_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_fhs_content=actual-fhs_content_extension
        actual=actual_before_fhs_content
        current_with_shared_library=actual
        shared_library_extension={name for name,_,_ in CURRENT_SHARED_LIBRARY_INPUTS}
        self.assertEqual(len(shared_library_extension),10)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SHARED_LIBRARY_INPUTS']
        self.assertTrue(shared_library_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_shared_library=actual-shared_library_extension
        actual=actual_before_shared_library
        current_with_presentation_paint=actual
        presentation_paint_extension={name for name,_,_ in CURRENT_PRESENTATION_PAINT_INPUTS}
        self.assertEqual(len(presentation_paint_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_PRESENTATION_PAINT_INPUTS']
        self.assertTrue(presentation_paint_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_presentation_paint=actual-presentation_paint_extension
        actual=actual_before_presentation_paint
        current_with_course_metadata=actual
        course_metadata_extension={name for name,_,_ in CURRENT_COURSE_METADATA_INPUTS}
        self.assertEqual(len(course_metadata_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_COURSE_METADATA_INPUTS']
        self.assertTrue(course_metadata_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_course_metadata=actual-course_metadata_extension
        actual=actual_before_course_metadata
        current_with_associative_lesson=actual
        associative_lesson_extension={name for name,_,_ in CURRENT_ASSOCIATIVE_LESSON_INPUTS}
        self.assertEqual(len(associative_lesson_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_ASSOCIATIVE_LESSON_INPUTS']
        self.assertTrue(associative_lesson_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_associative_lesson=actual-associative_lesson_extension
        actual=actual_before_associative_lesson
        current_with_associative=actual
        associative_extension={name for name,_,_ in CURRENT_ASSOCIATIVE_INPUTS}
        self.assertEqual(len(associative_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_ASSOCIATIVE_INPUTS']
        self.assertTrue(associative_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_associative=actual-associative_extension
        actual=actual_before_associative
        current_with_raid1=actual
        raid1_extension={name for name,_,_ in CURRENT_RAID1_INPUTS}
        self.assertEqual(len(raid1_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RAID1_INPUTS']
        self.assertTrue(raid1_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_raid1=actual-raid1_extension
        actual=actual_before_raid1
        current_with_debian_conffile=actual
        debian_conffile_extension={name for name,_,_ in CURRENT_DEBIAN_CONFFILE_INPUTS}
        self.assertEqual(len(debian_conffile_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEBIAN_CONFFILE_INPUTS']
        self.assertTrue(debian_conffile_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_debian_conffile=actual-debian_conffile_extension
        actual=actual_before_debian_conffile
        current_with_shell_file_lifecycle_lesson=actual
        shell_file_lifecycle_lesson_extension={name for name,_,_ in CURRENT_SHELL_FILE_LIFECYCLE_LESSON_INPUTS}
        self.assertEqual(len(shell_file_lifecycle_lesson_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SHELL_FILE_LIFECYCLE_LESSON_INPUTS']
        self.assertTrue(shell_file_lifecycle_lesson_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_shell_file_lifecycle_lesson=actual-shell_file_lifecycle_lesson_extension
        actual=actual_before_shell_file_lifecycle_lesson
        current_with_shell_file_lifecycle=actual
        shell_file_lifecycle_extension={name for name,_,_ in CURRENT_SHELL_FILE_LIFECYCLE_INPUTS}
        self.assertEqual(len(shell_file_lifecycle_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SHELL_FILE_LIFECYCLE_INPUTS']
        self.assertTrue(shell_file_lifecycle_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_shell_file_lifecycle=actual-shell_file_lifecycle_extension
        actual=actual_before_shell_file_lifecycle
        current_with_shell_startup=actual
        shell_startup_extension={name for name,_,_ in CURRENT_SHELL_STARTUP_INPUTS}
        self.assertEqual(len(shell_startup_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SHELL_STARTUP_INPUTS']
        self.assertTrue(shell_startup_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_shell_startup=actual-shell_startup_extension
        actual=actual_before_shell_startup
        current_with_fs_integrity=actual
        fs_integrity_extension={name for name,_,_ in CURRENT_FS_INTEGRITY_INPUTS}
        self.assertEqual(len(fs_integrity_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_FS_INTEGRITY_INPUTS']
        self.assertTrue(fs_integrity_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_fs_integrity=actual-fs_integrity_extension
        actual=actual_before_fs_integrity
        current_with_nm_dhcp=actual
        nm_dhcp_extension={name for name,_,_ in CURRENT_NM_DHCP_INPUTS}
        self.assertEqual(len(nm_dhcp_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_NM_DHCP_INPUTS']
        self.assertTrue(nm_dhcp_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_nm_dhcp=actual-nm_dhcp_extension
        actual=actual_before_nm_dhcp
        current_with_debian_conflicts=actual
        debian_conflicts_extension={name for name,_,_ in CURRENT_DEBIAN_CONFLICT_INPUTS}
        self.assertEqual(len(debian_conflicts_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEBIAN_CONFLICT_INPUTS']
        self.assertTrue(debian_conflicts_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_debian_conflicts=actual-debian_conflicts_extension
        actual=actual_before_debian_conflicts
        current_with_user_mounts=actual
        user_mounts_extension={name for name,_,_ in CURRENT_USER_MOUNTS_INPUTS}
        self.assertEqual(len(user_mounts_extension),8)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_USER_MOUNTS_INPUTS']
        self.assertTrue(user_mounts_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_user_mounts=actual-user_mounts_extension
        actual=actual_before_user_mounts
        current_with_shell_environment=actual
        shell_environment_extension={name for name,_,_ in CURRENT_SHELL_ENVIRONMENT_INPUTS}
        self.assertEqual(len(shell_environment_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SHELL_ENVIRONMENT_INPUTS']
        self.assertTrue(shell_environment_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_shell_environment=actual-shell_environment_extension
        actual=actual_before_shell_environment
        current_with_shell_tracing=actual
        shell_tracing_extension={name for name,_,_ in CURRENT_SHELL_TRACING_INPUTS}
        self.assertEqual(len(shell_tracing_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_SHELL_TRACING_INPUTS']
        self.assertTrue(shell_tracing_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_shell_tracing=actual-shell_tracing_extension
        actual=actual_before_shell_tracing
        current_with_debian_source_ui=actual
        debian_source_ui_extension={name for name,_,_ in CURRENT_DEBIAN_SOURCE_UI_INPUTS}
        self.assertEqual(len(debian_source_ui_extension),3)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEBIAN_SOURCE_UI_INPUTS']
        self.assertTrue(debian_source_ui_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_debian_source_ui=actual-debian_source_ui_extension
        actual=actual_before_debian_source_ui
        current_with_debian_source_content=actual
        debian_source_content_extension={name for name,_,_ in CURRENT_DEBIAN_SOURCE_CONTENT_INPUTS}
        self.assertEqual(len(debian_source_content_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEBIAN_SOURCE_CONTENT_INPUTS']
        self.assertTrue(debian_source_content_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_debian_source_content=actual-debian_source_content_extension
        actual=actual_before_debian_source_content
        current_with_debian_source_workspace=actual
        debian_source_workspace_extension={name for name,_,_ in CURRENT_DEBIAN_SOURCE_WORKSPACE_INPUTS}
        self.assertEqual(len(debian_source_workspace_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEBIAN_SOURCE_WORKSPACE_INPUTS']
        self.assertTrue(debian_source_workspace_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_debian_source_workspace=actual-debian_source_workspace_extension
        actual=actual_before_debian_source_workspace
        current_with_debian_source_selection=actual
        debian_source_selection_extension={name for name,_,_ in CURRENT_DEBIAN_SOURCE_SELECTION_INPUTS}
        self.assertEqual(len(debian_source_selection_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEBIAN_SOURCE_SELECTION_INPUTS']
        self.assertTrue(debian_source_selection_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_debian_source_selection=actual-debian_source_selection_extension
        actual=actual_before_debian_source_selection
        current_with_debian_content_ui=actual
        debian_content_ui_extension={name for name,_,_ in CURRENT_DEBIAN_CONTENT_UI_INPUTS}
        self.assertEqual(len(debian_content_ui_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEBIAN_CONTENT_UI_INPUTS']
        self.assertTrue(debian_content_ui_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_debian_content_ui=actual-debian_content_ui_extension
        actual=actual_before_debian_content_ui
        current_with_debian_content=actual
        debian_content_extension={name for name,_,_ in CURRENT_DEBIAN_CONTENT_INPUTS}
        self.assertEqual(len(debian_content_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEBIAN_CONTENT_INPUTS']
        self.assertTrue(debian_content_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_debian_content=actual-debian_content_extension
        actual=actual_before_debian_content
        current_with_debian_contents=actual
        debian_contents_extension={name for name,_,_ in CURRENT_DEBIAN_CONTENTS_INPUTS}
        self.assertEqual(len(debian_contents_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEBIAN_CONTENTS_INPUTS']
        self.assertTrue(debian_contents_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_debian_contents=actual-debian_contents_extension
        actual=actual_before_debian_contents
        current_with_resolver_ui=actual
        resolver_ui_extension={name for name,_,_ in CURRENT_RESOLVER_UI_INPUTS}
        self.assertEqual(len(resolver_ui_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RESOLVER_UI_INPUTS']
        self.assertTrue(resolver_ui_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_resolver_ui=actual-resolver_ui_extension
        actual=actual_before_resolver_ui
        current_with_resolver_content=actual
        resolver_content_extension={name for name,_,_ in CURRENT_RESOLVER_CONTENT_INPUTS}
        self.assertEqual(len(resolver_content_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RESOLVER_CONTENT_INPUTS']
        self.assertTrue(resolver_content_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_resolver_content=actual-resolver_content_extension
        actual=actual_before_resolver_content
        current_with_resolver_search=actual
        resolver_search_extension={name for name,_,_ in CURRENT_RESOLVER_SEARCH_INPUTS}
        self.assertEqual(len(resolver_search_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_RESOLVER_SEARCH_INPUTS']
        self.assertTrue(resolver_search_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_resolver_search=actual-resolver_search_extension
        actual=actual_before_resolver_search
        current_with_client_name_lessons=actual
        client_name_lessons_extension={name for name,_,_ in CURRENT_CLIENT_NAME_LESSONS_INPUTS}
        self.assertEqual(len(client_name_lessons_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_CLIENT_NAME_LESSONS_INPUTS']
        self.assertTrue(client_name_lessons_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_client_name_lessons=actual-client_name_lessons_extension
        actual=actual_before_client_name_lessons
        current_with_client_names_workspace=actual
        client_names_workspace_extension={name for name,_,_ in CURRENT_CLIENT_NAMES_WORKSPACE_INPUTS}
        self.assertEqual(len(client_names_workspace_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_CLIENT_NAMES_WORKSPACE_INPUTS']
        self.assertTrue(client_names_workspace_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_client_names_workspace=actual-client_names_workspace_extension
        actual=actual_before_client_names_workspace
        current_with_client_names=actual
        client_names_extension={name for name,_,_ in CURRENT_CLIENT_NAMES_INPUTS}
        self.assertEqual(len(client_names_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_CLIENT_NAMES_INPUTS']
        self.assertTrue(client_names_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_client_names=actual-client_names_extension
        actual=actual_before_client_names
        current_with_removable_lesson=actual
        removable_lesson_extension={name for name,_,_ in CURRENT_REMOVABLE_LESSON_INPUTS}
        self.assertEqual(len(removable_lesson_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_REMOVABLE_LESSON_INPUTS']
        self.assertTrue(removable_lesson_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_removable_lesson=actual-removable_lesson_extension
        actual=actual_before_removable_lesson
        current_with_removable_workspace=actual
        removable_workspace_extension={name for name,_,_ in CURRENT_REMOVABLE_WORKSPACE_INPUTS}
        self.assertEqual(len(removable_workspace_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_REMOVABLE_WORKSPACE_INPUTS']
        self.assertTrue(removable_workspace_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_removable_workspace=actual-removable_workspace_extension
        actual=actual_before_removable_workspace
        current_with_removable_mount=actual
        removable_mount_extension={name for name,_,_ in CURRENT_REMOVABLE_MOUNT_INPUTS}
        self.assertEqual(len(removable_mount_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_REMOVABLE_MOUNT_INPUTS']
        self.assertTrue(removable_mount_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_removable_mount=actual-removable_mount_extension
        actual=actual_before_removable_mount
        current_with_nm_workspace=actual
        nm_workspace_extension={name for name,_,_ in CURRENT_NM_WORKSPACE_INPUTS}
        self.assertEqual(len(nm_workspace_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_NM_WORKSPACE_INPUTS']
        self.assertTrue(nm_workspace_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_nm_workspace=actual-nm_workspace_extension
        actual=actual_before_nm_workspace
        current_with_nm_policy=actual
        nm_policy_extension={name for name,_,_ in CURRENT_NM_POLICY_INPUTS}
        self.assertEqual(len(nm_policy_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_NM_POLICY_INPUTS']
        self.assertTrue(nm_policy_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_nm_policy=actual-nm_policy_extension
        actual=actual_before_nm_policy
        current_with_nm_profile=actual
        nm_profile_extension={name for name,_,_ in CURRENT_NM_PROFILE_INPUTS}
        self.assertEqual(len(nm_profile_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_NM_PROFILE_INPUTS']
        self.assertTrue(nm_profile_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_nm_profile=actual-nm_profile_extension
        actual=actual_before_nm_profile
        current_with_dhcp_workspace=actual
        dhcp_workspace_extension={name for name,_,_ in CURRENT_DHCP_WORKSPACE_INPUTS}
        self.assertEqual(len(dhcp_workspace_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DHCP_WORKSPACE_INPUTS']
        self.assertTrue(dhcp_workspace_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_dhcp_workspace=actual-dhcp_workspace_extension
        actual=actual_before_dhcp_workspace
        current_with_dhcp_config=actual
        dhcp_config_extension={name for name,_,_ in CURRENT_DHCP_CONFIG_INPUTS}
        self.assertEqual(len(dhcp_config_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DHCP_CONFIG_INPUTS']
        self.assertTrue(dhcp_config_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_dhcp_config=actual-dhcp_config_extension
        actual=actual_before_dhcp_config
        current_with_backup_selection=actual
        backup_selection_extension={name for name,_,_ in CURRENT_BACKUP_SELECTION_INPUTS}
        self.assertEqual(len(backup_selection_extension),5)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_BACKUP_SELECTION_INPUTS']
        self.assertTrue(backup_selection_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_backup_selection=actual-backup_selection_extension
        actual=actual_before_backup_selection
        current_with_qos_ui=actual
        qos_ui_extension={name for name,_,_ in CURRENT_QOS_UI_INPUTS}
        self.assertEqual(len(qos_ui_extension),3)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_QOS_UI_INPUTS']
        self.assertTrue(qos_ui_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_qos_ui=actual-qos_ui_extension
        actual=actual_before_qos_ui
        current_with_qos_authored=actual
        qos_authored_extension={name for name,_,_ in CURRENT_QOS_AUTHORED_INPUTS}
        self.assertEqual(len(qos_authored_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_QOS_AUTHORED_INPUTS']
        self.assertTrue(qos_authored_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_qos_authored=actual-qos_authored_extension
        actual=actual_before_qos_authored
        current_with_qos=actual
        qos_extension={name for name,_,_ in CURRENT_QOS_FOUNDATION_INPUTS}
        self.assertEqual(len(qos_extension),2)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_QOS_FOUNDATION_INPUTS']
        self.assertTrue(qos_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_qos=actual-qos_extension
        actual=actual_before_qos
        current_with_startup=actual
        startup_extension={name for name,_,_ in CURRENT_STARTUP_LESSONS_INPUTS}
        self.assertEqual(len(startup_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_STARTUP_LESSONS_INPUTS']
        self.assertTrue(startup_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_startup=actual-startup_extension
        actual=actual_before_startup
        current_with_linux_logging=actual
        linux_logging_extension={name for name,_,_ in CURRENT_LINUX_LOGGING_INPUTS}
        self.assertEqual(len(linux_logging_extension),7)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LINUX_LOGGING_INPUTS']
        self.assertTrue(linux_logging_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_linux_logging=actual-linux_logging_extension
        actual=actual_before_linux_logging
        current_with_debug_recovery=actual
        debug_recovery_extension={name for name,_,_ in CURRENT_DEBUG_RECOVERY_INPUTS}
        self.assertEqual(len(debug_recovery_extension),4)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_DEBUG_RECOVERY_INPUTS']
        self.assertTrue(debug_recovery_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_debug_recovery=actual-debug_recovery_extension
        actual=actual_before_debug_recovery
        current_with_host_lessons=actual
        host_lesson_extension={name for name,_,_ in CURRENT_HOST_LESSONS_INPUTS}
        self.assertEqual(len(host_lesson_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_HOST_LESSONS_INPUTS']
        self.assertTrue(host_lesson_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_host_lessons=actual-host_lesson_extension
        actual=actual_before_host_lessons
        current_with_host=actual
        host_extension={name for name,_,_ in CURRENT_HOST_FOUNDATION_INPUTS}
        self.assertEqual(len(host_extension),1)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_HOST_FOUNDATION_INPUTS']
        self.assertTrue(host_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_host=actual-host_extension
        actual=actual_before_host
        current_with_repositories=actual
        repository_extension={name for name,_,_ in CURRENT_REPOSITORY_LESSONS_INPUTS}
        self.assertEqual(len(repository_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_REPOSITORY_LESSONS_INPUTS']
        self.assertTrue(repository_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_repositories=actual-repository_extension
        actual=actual_before_repositories
        current_with_capacity=actual
        capacity_extension={name for name,_,_ in CURRENT_CAPACITY_LESSONS_INPUTS}
        self.assertEqual(len(capacity_extension),6)
        others=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_CAPACITY_LESSONS_INPUTS']
        self.assertTrue(capacity_extension.isdisjoint(set(c.COMPILED)|{r[0]for table in others for r in table}))
        actual_before_capacity=actual-capacity_extension
        actual=actual_before_capacity
        current_with_linux = actual
        linux_extension = {name for name,_,_ in CURRENT_LINUX_FOUNDATIONS_INPUTS}
        self.assertEqual(len(linux_extension),2)
        other_tables=[v for k,v in globals().items()if k.startswith('CURRENT_')and k!='CURRENT_LINUX_FOUNDATIONS_INPUTS']
        known={row[0]for table in other_tables for row in table}
        self.assertTrue(linux_extension.isdisjoint(set(c.COMPILED)|known))
        actual_before_linux=actual-linux_extension
        actual=actual_before_linux
        current_with_teaching = actual
        teaching_extension = {name for name, _, _ in CURRENT_TEACHING_LINKS_INPUTS}
        self.assertEqual(len(teaching_extension), 30)
        other_current = [value for key,value in globals().items() if key.startswith('CURRENT_') and key != 'CURRENT_TEACHING_LINKS_INPUTS']
        known_current = {row[0] for table in other_current for row in table}
        self.assertTrue(teaching_extension.isdisjoint(set(c.COMPILED) | known_current))
        actual_before_teaching = actual - teaching_extension
        actual = actual_before_teaching
        snapshot_native_extension = {name for name, _, _ in CURRENT_SNAPSHOT_NATIVE_INPUTS}
        self.assertEqual(len(snapshot_native_extension), 2)
        actual_with_snapshot = actual
        actual_before_snapshot = actual - snapshot_native_extension
        actual = actual_before_snapshot
        listed = {name for name in c.COMPILED if Path(name).parts[0] in c.SOURCE_FAMILIES}
        current_before_authored_syslog = actual
        authored_syslog_extension = {name for name, _, _ in CURRENT_AUTHORED_SYSLOG_INPUTS}
        self.assertEqual(len(authored_syslog_extension), 1)
        self.assertTrue(listed.isdisjoint(authored_syslog_extension))
        admitted_before_authored_syslog = current_before_authored_syslog - authored_syslog_extension
        actual = admitted_before_authored_syslog
        current_before_syslog = actual
        syslog_extension = {name for name, _, _ in CURRENT_SYSLOG_WORKSPACE_INPUTS}
        self.assertEqual(len(syslog_extension), 5)
        self.assertTrue(listed.isdisjoint(syslog_extension))
        admitted_before_syslog = current_before_syslog - syslog_extension
        actual = admitted_before_syslog
        extension = {name for name, _, _ in CURRENT_NATIVE_QA_INPUTS}
        self.assertEqual(len(extension), 2)
        self.assertTrue(listed.isdisjoint(extension))
        storage_extension = {name for name, _, _ in CURRENT_STORAGE_WORKSPACE_INPUTS}
        self.assertEqual(len(storage_extension), 5)
        self.assertTrue((listed | extension).isdisjoint(storage_extension))
        authored_extension = {name for name, _, _ in CURRENT_AUTHORED_STORAGE_INPUTS}
        self.assertEqual(len(authored_extension), 1)
        self.assertTrue((listed | extension | storage_extension).isdisjoint(authored_extension))
        storage_native_extension = {name for name, _, _ in CURRENT_STORAGE_NATIVE_INPUTS}
        self.assertEqual(len(storage_native_extension), 2)
        self.assertTrue((listed | extension | storage_extension | authored_extension).isdisjoint(storage_native_extension))
        literal_extension = {name for name, _, _ in CURRENT_LITERAL_PRODUCT_INPUTS}
        self.assertEqual(len(literal_extension), 1)
        self.assertTrue((listed | extension | storage_extension | authored_extension | storage_native_extension).isdisjoint(literal_extension))
        literal_authored_extension = {name for name, _, _ in CURRENT_LITERAL_AUTHORED_INPUTS}
        self.assertEqual(len(literal_authored_extension), 2)
        self.assertTrue((listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension).isdisjoint(literal_authored_extension))
        hsrp_extension = {name for name, _, _ in CURRENT_HSRP_WORKSPACE_INPUTS}
        self.assertEqual(len(hsrp_extension),5)
        self.assertTrue((listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension | literal_authored_extension).isdisjoint(hsrp_extension))
        hsrp_authored_extension = {name for name, _, _ in CURRENT_AUTHORED_HSRP_INPUTS}
        self.assertEqual(len(hsrp_authored_extension),1)
        self.assertTrue((listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension | literal_authored_extension | hsrp_extension).isdisjoint(hsrp_authored_extension))
        literal_native_extension = {name for name, _, _ in CURRENT_LITERAL_NATIVE_INPUTS}
        self.assertEqual(len(literal_native_extension), 2)
        self.assertTrue((listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension | literal_authored_extension | hsrp_extension | hsrp_authored_extension).isdisjoint(literal_native_extension))
        actual_before_hsrp = actual - hsrp_extension - hsrp_authored_extension - literal_native_extension
        actual_before_literal = actual_before_hsrp - literal_extension - literal_authored_extension
        # Project each complete predecessor by removing ALL fixed later sets.
        # The final equality still rejects every unknown, omitted or extra path.
        self.assertEqual(listed | extension, actual_before_literal - storage_extension - authored_extension - storage_native_extension)
        self.assertEqual(listed | extension | storage_extension, actual_before_literal - authored_extension - storage_native_extension)
        self.assertEqual(listed | extension | storage_extension | authored_extension, actual_before_literal - storage_native_extension)
        self.assertEqual(listed | extension | storage_extension | authored_extension | storage_native_extension, actual_before_literal)
        self.assertEqual(listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension, actual_before_hsrp - literal_authored_extension)
        self.assertEqual(listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension | literal_authored_extension, actual_before_hsrp)
        self.assertEqual(listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension | literal_authored_extension | hsrp_extension, actual - hsrp_authored_extension - literal_native_extension)
        self.assertEqual(listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension | literal_authored_extension | hsrp_extension | hsrp_authored_extension, actual - literal_native_extension)
        self.assertEqual(listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension | literal_authored_extension | hsrp_extension | hsrp_authored_extension | literal_native_extension, actual)
        for name, size, sha in CURRENT_LITERAL_NATIVE_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name,size,sha in CURRENT_AUTHORED_HSRP_INPUTS:
            path=PROJECT/name
            info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))
        for name, size, sha in CURRENT_HSRP_WORKSPACE_INPUTS:
            path=PROJECT/name
            info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))
        for name, size, sha in CURRENT_LITERAL_AUTHORED_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name, size, sha in CURRENT_LITERAL_PRODUCT_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name, size, sha in CURRENT_STORAGE_NATIVE_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name, size, sha in CURRENT_AUTHORED_STORAGE_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name, size, sha in CURRENT_STORAGE_WORKSPACE_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name, size, sha in CURRENT_NATIVE_QA_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        self.assertTrue({'Cargo.toml', 'Cargo.lock', c.PROBE_SOURCE,
                         'examples/support/automation_digest.rs', c.FIXTURE} <= set(c.COMPILED))
        self.assertEqual(len(c.CORE_PINS), 18)
        self.assertEqual(c.CORE_PINS['src/shell/grep.rs'],
                         'f0b554b0fcbcbdf99e3898e7402f48749be2b01e110398ec32a46f26e1fd7863')

        self.assertTrue(actual_before_snapshot.isdisjoint(snapshot_native_extension))
        self.assertEqual(actual_with_snapshot, actual_before_snapshot | snapshot_native_extension)
        for name, size, sha in CURRENT_SNAPSHOT_NATIVE_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual((info.st_size, c.digest(path.read_bytes())), (size, sha))

        # Retain every original predecessor equation above. Close this exact
        # new layer disjointly against the actual entire current inventory.
        self.assertTrue(admitted_before_syslog.isdisjoint(syslog_extension))
        self.assertEqual(current_before_syslog, admitted_before_syslog | syslog_extension)
        for name, size, sha in CURRENT_SYSLOG_WORKSPACE_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual((info.st_size, c.digest(path.read_bytes())), (size, sha))

        # Retain every original predecessor equation above. Close this exact
        # new layer disjointly against the actual entire current inventory.
        self.assertTrue(admitted_before_authored_syslog.isdisjoint(authored_syslog_extension))
        self.assertEqual(current_before_authored_syslog, admitted_before_authored_syslog | authored_syslog_extension)
        for name, size, sha in CURRENT_AUTHORED_SYSLOG_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual((info.st_size, c.digest(path.read_bytes())), (size, sha))

        self.assertEqual(current_with_teaching, actual_before_teaching | teaching_extension)
        for name,size,sha in CURRENT_TEACHING_LINKS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_host_lessons,actual_before_host_lessons|host_lesson_extension)
        for name,size,sha in CURRENT_HOST_LESSONS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_campus_architecture,actual_before_campus_architecture|campus_architecture_extension)
        for name,size,sha in CURRENT_CAMPUS_ARCHITECTURE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_leaf_spine,actual_before_leaf_spine|leaf_spine_extension)
        for name,size,sha in CURRENT_LEAF_SPINE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ospf_broadcast_foundation,actual_before_ospf_broadcast_foundation|ospf_broadcast_foundation_extension)
        for name,size,sha in CURRENT_OSPF_BROADCAST_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_route_reading_foundation,actual_before_route_reading_foundation|route_reading_foundation_extension)
        for name,size,sha in CURRENT_ROUTE_READING_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_nfs_export_foundation,actual_before_nfs_export_foundation|nfs_export_foundation_extension)
        for name,size,sha in CURRENT_NFS_EXPORT_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_vlan_gateway_foundation,actual_before_vlan_gateway_foundation|vlan_gateway_foundation_extension)
        for name,size,sha in CURRENT_VLAN_GATEWAY_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_host_dns_security_foundation,actual_before_host_dns_security_foundation|host_dns_security_foundation_extension)
        for name,size,sha in CURRENT_HOST_DNS_SECURITY_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_site_connectivity_foundation,actual_before_site_connectivity_foundation|site_connectivity_foundation_extension)
        for name,size,sha in CURRENT_SITE_CONNECTIVITY_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_network_evidence_review_foundation,actual_before_network_evidence_review_foundation|network_evidence_review_foundation_extension)
        for name,size,sha in CURRENT_NETWORK_EVIDENCE_REVIEW_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_staged_network_foundation,actual_before_staged_network_foundation|staged_network_foundation_extension)
        for name,size,sha in CURRENT_STAGED_NETWORK_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_credentials_vpn_foundation,actual_before_credentials_vpn_foundation|credentials_vpn_foundation_extension)
        for name,size,sha in CURRENT_CREDENTIALS_VPN_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_security_investigation_foundation,actual_before_security_investigation_foundation|security_investigation_foundation_extension)
        for name,size,sha in CURRENT_SECURITY_INVESTIGATION_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_desktop_profile_foundation,actual_before_desktop_profile_foundation|desktop_profile_foundation_extension)
        for name,size,sha in CURRENT_DESKTOP_PROFILE_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_apache_https_foundation,actual_before_apache_https_foundation|apache_https_foundation_extension)
        for name,size,sha in CURRENT_APACHE_HTTPS_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ata_smart_foundation,actual_before_ata_smart_foundation|ata_smart_foundation_extension)
        for name,size,sha in CURRENT_ATA_SMART_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_iscsi_foundation,actual_before_iscsi_foundation|iscsi_foundation_extension)
        for name,size,sha in CURRENT_ISCSI_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_pxe_foundation,actual_before_pxe_foundation|pxe_foundation_extension)
        for name,size,sha in CURRENT_PXE_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_initramfs_foundation,actual_before_initramfs_foundation|initramfs_foundation_extension)
        for name,size,sha in CURRENT_INITRAMFS_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_dkms_foundation,actual_before_dkms_foundation|dkms_foundation_extension)
        for name,size,sha in CURRENT_DKMS_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_kernel_build_foundation,actual_before_kernel_build_foundation|kernel_build_foundation_extension)
        for name,size,sha in CURRENT_KERNEL_BUILD_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_poe,actual_before_poe|poe_extension)
        for name,size,sha in CURRENT_POE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_telnet_transport,actual_before_telnet_transport|telnet_transport_extension)
        for name,size,sha in CURRENT_TELNET_TRANSPORT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_nc_probe,actual_before_nc_probe|nc_probe_extension)
        for name,size,sha in CURRENT_NC_PROBE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ids_rule,actual_before_ids_rule|ids_rule_extension)
        for name,size,sha in CURRENT_IDS_RULE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_log_filter,actual_before_log_filter|log_filter_extension)
        for name,size,sha in CURRENT_LOG_FILTER_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_rpm_configuration,actual_before_rpm_configuration|rpm_configuration_extension)
        for name,size,sha in CURRENT_RPM_CONFIGURATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_vpn_point,actual_before_vpn_point|vpn_point_extension)
        for name,size,sha in CURRENT_VPN_POINT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_service_exposure,actual_before_service_exposure|service_exposure_extension)
        for name,size,sha in CURRENT_SERVICE_EXPOSURE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))
        self.assertEqual(current_with_ftp_identity,actual_before_ftp_identity|ftp_identity_extension)
        for name,size,sha in CURRENT_FTP_IDENTITY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))
        self.assertEqual(current_with_linux_v6_startup,actual_before_linux_v6_startup|linux_v6_startup_extension)
        for name,size,sha in CURRENT_LINUX_V6_STARTUP_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))
        self.assertEqual(current_with_linux_v6_admin,actual_before_linux_v6_admin|linux_v6_admin_extension)
        for name,size,sha in CURRENT_LINUX_V6_ADMIN_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))
        self.assertEqual(current_with_linux_service_rules,actual_before_linux_service_rules|linux_service_rules_extension)
        for name,size,sha in CURRENT_LINUX_SERVICE_RULE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))
        self.assertEqual(current_with_linux_ipv6,actual_before_linux_ipv6|linux_ipv6_extension)
        for name,size,sha in CURRENT_LINUX_IPV6_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))
        self.assertEqual(current_with_linux_connections,actual_before_linux_connections|linux_connections_extension)
        for name,size,sha in CURRENT_LINUX_CONNECTION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_linux_router,actual_before_linux_router|linux_router_extension)
        for name,size,sha in CURRENT_LINUX_ROUTER_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_mailbox_admin,actual_before_mailbox_admin|mailbox_admin_extension)
        for name,size,sha in CURRENT_MAILBOX_ADMIN_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_mailbox_access,actual_before_mailbox_access|mailbox_access_extension)
        for name,size,sha in CURRENT_MAILBOX_ACCESS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_procmail_migration,actual_before_procmail_migration|procmail_migration_extension)
        for name,size,sha in CURRENT_PROCMAIL_MIGRATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_sieve_vacation,actual_before_sieve_vacation|sieve_vacation_extension)
        for name,size,sha in CURRENT_SIEVE_VACATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_sieve_delivery,actual_before_sieve_delivery|sieve_delivery_extension)
        for name,size,sha in CURRENT_SIEVE_DELIVERY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_sieve_address,actual_before_sieve_address|sieve_address_extension)
        for name,size,sha in CURRENT_SIEVE_ADDRESS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_sieve,actual_before_sieve|sieve_extension)
        for name,size,sha in CURRENT_SIEVE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_smtp,actual_before_smtp|smtp_extension)
        for name,size,sha in CURRENT_SMTP_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_guest_image,actual_before_guest_image|guest_image_extension)
        for name,size,sha in CURRENT_GUEST_IMAGE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_virtual_isolation,actual_before_virtual_isolation|virtual_isolation_extension)
        for name,size,sha in CURRENT_VIRTUAL_ISOLATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_samba_share,actual_before_samba_share|samba_share_extension)
        for name,size,sha in CURRENT_SAMBA_SHARE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_network_transfer,actual_before_network_transfer|network_transfer_extension)
        for name,size,sha in CURRENT_NETWORK_TRANSFER_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_handover_replay,actual_before_handover_replay|handover_replay_extension)
        for name,size,sha in CURRENT_HANDOVER_REPLAY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_editor_handover,actual_before_editor_handover|editor_handover_extension)
        for name,size,sha in CURRENT_EDITOR_HANDOVER_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_modal_editing,actual_before_modal_editing|modal_editing_extension)
        for name,size,sha in CURRENT_MODAL_EDITING_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ssh_access,actual_before_ssh_access|ssh_access_extension)
        for name,size,sha in CURRENT_SSH_ACCESS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_dns_query_tools,actual_before_dns_query_tools|dns_query_tools_extension)
        for name,size,sha in CURRENT_DNS_QUERY_TOOLS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_direct_dns,actual_before_direct_dns|direct_dns_extension)
        for name,size,sha in CURRENT_DIRECT_DNS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_client_diagnostic,actual_before_client_diagnostic|client_diagnostic_extension)
        for name,size,sha in CURRENT_CLIENT_DIAGNOSTIC_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_rest_controller,actual_before_rest_controller|rest_controller_extension)
        for name,size,sha in CURRENT_REST_CONTROLLER_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_host_echo,actual_before_host_echo|host_echo_extension)
        for name,size,sha in CURRENT_HOST_ECHO_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_protocol_evidence,actual_before_protocol_evidence|protocol_evidence_extension)
        for name,size,sha in CURRENT_PROTOCOL_EVIDENCE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_mixed_ip,actual_before_mixed_ip|mixed_ip_extension)
        for name,size,sha in CURRENT_MIXED_IP_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_service_names,actual_before_service_names|service_names_extension)
        for name,size,sha in CURRENT_SERVICE_NAMES_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_reverse_proxy,actual_before_reverse_proxy|reverse_proxy_extension)
        for name,size,sha in CURRENT_REVERSE_PROXY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_http_cache,actual_before_http_cache|http_cache_extension)
        for name,size,sha in CURRENT_HTTP_CACHE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_proxy_policy,actual_before_proxy_policy|proxy_policy_extension)
        for name,size,sha in CURRENT_PROXY_POLICY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_snmp,actual_before_snmp|snmp_extension)
        for name,size,sha in CURRENT_SNMP_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_wlan_paths,actual_before_wlan_paths|wlan_paths_extension)
        for name,size,sha in CURRENT_WLAN_PATH_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_wlan_identity,actual_before_wlan_identity|wlan_identity_extension)
        for name,size,sha in CURRENT_WLAN_IDENTITY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_wireless_channels,actual_before_wireless_channels|wireless_channels_extension)
        for name,size,sha in CURRENT_WIRELESS_CHANNEL_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_timezones,actual_before_timezones|timezones_extension)
        for name,size,sha in CURRENT_TIMEZONES_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_localization,actual_before_localization|localization_extension)
        for name,size,sha in CURRENT_LOCALIZATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_print_queues,actual_before_print_queues|print_queues_extension)
        for name,size,sha in CURRENT_PRINT_QUEUES_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_dns_access,actual_before_dns_access|dns_access_extension)
        for name,size,sha in CURRENT_DNS_ACCESS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_source_install,actual_before_source_install|source_install_extension)
        for name,size,sha in CURRENT_SOURCE_INSTALL_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_maintenance_notice,actual_before_maintenance_notice|maintenance_notice_extension)
        for name,size,sha in CURRENT_MAINTENANCE_NOTICE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_capacity_forecast,actual_before_capacity_forecast|capacity_forecast_extension)
        for name,size,sha in CURRENT_CAPACITY_FORECAST_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_resource_observation,actual_before_resource_observation|resource_observation_extension)
        for name,size,sha in CURRENT_RESOURCE_OBSERVATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_resource_measurement,actual_before_resource_measurement|resource_measurement_extension)
        for name,size,sha in CURRENT_RESOURCE_MEASUREMENT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_layout_policy,actual_before_layout_policy|layout_policy_extension)
        for name,size,sha in CURRENT_LAYOUT_POLICY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_json_inventory,actual_before_json_inventory|json_inventory_extension)
        for name,size,sha in CURRENT_JSON_INVENTORY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_wired_device,actual_before_wired_device|wired_device_extension)
        for name,size,sha in CURRENT_WIRED_DEVICE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_local_mail,actual_before_local_mail|local_mail_extension)
        for name,size,sha in CURRENT_LOCAL_MAIL_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_copied_journal,actual_before_copied_journal|copied_journal_extension)
        for name,size,sha in CURRENT_COPIED_JOURNAL_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_journal_archive,actual_before_journal_archive|journal_archive_extension)
        for name,size,sha in CURRENT_COPIED_JOURNAL_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_chrony_source,actual_before_chrony_source|chrony_source_extension)
        for name,size,sha in CURRENT_CHRONY_SOURCE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_rescue_startup,actual_before_rescue_startup|rescue_startup_extension)
        for name,size,sha in CURRENT_RESCUE_STARTUP_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_grub_once,actual_before_grub_once|grub_once_extension)
        for name,size,sha in CURRENT_GRUB_ONCE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_efi_removable,actual_before_efi_removable|efi_removable_extension)
        for name,size,sha in CURRENT_EFI_REMOVABLE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_efi_loader,actual_before_efi_loader|efi_loader_extension)
        for name,size,sha in CURRENT_EFI_LOADER_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_bios_loader,actual_before_bios_loader|bios_loader_extension)
        for name,size,sha in CURRENT_BIOS_LOADER_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_grub_settings,actual_before_grub_settings|grub_settings_extension)
        for name,size,sha in CURRENT_GRUB_SETTINGS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_grub_menu,actual_before_grub_menu|grub_menu_extension)
        for name,size,sha in CURRENT_GRUB_MENU_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_boot_entry,actual_before_boot_entry|boot_entry_extension)
        for name,size,sha in CURRENT_BOOT_ENTRY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_eui64,actual_before_eui64|eui64_extension)
        for name,size,sha in CURRENT_EUI64_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ldap_schema,actual_before_ldap_schema|ldap_schema_extension)
        for name,size,sha in CURRENT_LDAP_SCHEMA_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ldap_configuration,actual_before_ldap_configuration|ldap_configuration_extension)
        for name,size,sha in CURRENT_LDAP_CONFIGURATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ldap_access,actual_before_ldap_access|ldap_access_extension)
        for name,size,sha in CURRENT_LDAP_ACCESS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ldap_protection,actual_before_ldap_protection|ldap_protection_extension)
        for name,size,sha in CURRENT_LDAP_PROTECTION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ldap_delete,actual_before_ldap_delete|ldap_delete_extension)
        for name,size,sha in CURRENT_LDAP_DELETE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ldap_utility,actual_before_ldap_utility|ldap_utility_extension)
        for name,size,sha in CURRENT_LDAP_UTILITY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ldap_changes,actual_before_ldap_changes|ldap_changes_extension)
        for name,size,sha in CURRENT_LDAP_CHANGE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_ldap_query,actual_before_ldap_query|ldap_query_extension)
        for name,size,sha in CURRENT_LDAP_QUERY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_aaa_methods,actual_before_aaa_methods|aaa_methods_extension)
        for name,size,sha in CURRENT_AAA_METHODS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_local_access,actual_before_local_access|local_access_extension)
        for name,size,sha in CURRENT_LOCAL_ACCESS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_management_policy,actual_before_management_policy|management_policy_extension)
        for name,size,sha in CURRENT_MANAGEMENT_POLICY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_arp_inspection,actual_before_arp_inspection|arp_inspection_extension)
        for name,size,sha in CURRENT_ARP_INSPECTION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_dhcp_snooping,actual_before_dhcp_snooping|dhcp_snooping_extension)
        for name,size,sha in CURRENT_DHCP_SNOOPING_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_access_outputs,actual_before_access_outputs|access_outputs_extension)
        for name,size,sha in CURRENT_ACCESS_OUTPUTS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_port_security,actual_before_port_security|port_security_extension)
        for name,size,sha in CURRENT_PORT_SECURITY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_static_cam,actual_before_static_cam|static_cam_extension)
        for name,size,sha in CURRENT_STATIC_CAM_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_aging,actual_before_aging|aging_extension)
        for name,size,sha in CURRENT_AGING_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_pam_control,actual_before_pam_control|pam_control_extension)
        for name,size,sha in CURRENT_PAM_CONTROL_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_bridge_foundation,actual_before_bridge_foundation|bridge_foundation_extension)
        for name,size,sha in CURRENT_BRIDGE_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_bond_control,actual_before_bond_control|bond_control_extension)
        for name,size,sha in CURRENT_BOND_CONTROL_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_vlan_host,actual_before_vlan_host|vlan_host_extension)
        for name,size,sha in CURRENT_VLAN_HOST_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_raid_assembly,actual_before_raid_assembly|raid_assembly_extension)
        for name,size,sha in CURRENT_RAID_ASSEMBLY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_autofs,actual_before_autofs|autofs_extension)
        for name,size,sha in CURRENT_AUTOFS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_lvm_mount,actual_before_lvm_mount|lvm_mount_extension)
        for name,size,sha in CURRENT_LVM_MOUNT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_lvm_config_scan,actual_before_lvm_config_scan|lvm_config_scan_extension)
        for name,size,sha in CURRENT_LVM_CONFIG_SCAN_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_device_path_model,actual_before_device_path_model|device_path_model_extension)
        for name,size,sha in CURRENT_DEVICE_PATH_MODEL_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_cow_model,actual_before_cow_model|cow_model_extension)
        for name,size,sha in CURRENT_COW_MODEL_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_mbr_layout,actual_before_mbr_layout|mbr_layout_extension)
        for name,size,sha in CURRENT_MBR_LAYOUT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_pv_capacity,actual_before_pv_capacity|pv_capacity_extension)
        for name,size,sha in CURRENT_PV_CAPACITY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_lvm_rename,actual_before_lvm_rename|lvm_rename_extension)
        for name,size,sha in CURRENT_LVM_RENAME_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_lvm_activation,actual_before_lvm_activation|lvm_activation_extension)
        for name,size,sha in CURRENT_LVM_ACTIVATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_lvm_retirement,actual_before_lvm_retirement|lvm_retirement_extension)
        for name,size,sha in CURRENT_LVM_RETIREMENT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_lvm_provisioning,actual_before_lvm_provisioning|lvm_provisioning_extension)
        for name,size,sha in CURRENT_LVM_PROVISIONING_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_swap_policy,actual_before_swap_policy|swap_policy_extension)
        for name,size,sha in CURRENT_SWAP_POLICY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_host_ipv6,actual_before_host_ipv6|host_ipv6_extension)
        for name,size,sha in CURRENT_HOST_IPV6_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_rpm_metadata,actual_before_rpm_metadata|rpm_metadata_extension)
        for name,size,sha in CURRENT_RPM_METADATA_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_log_triage,actual_before_log_triage|log_triage_extension)
        for name,size,sha in CURRENT_LOG_TRIAGE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_kernel_names,actual_before_kernel_names|kernel_names_extension)
        for name,size,sha in CURRENT_KERNEL_NAMES_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_kernel_module,actual_before_kernel_module|kernel_module_extension)
        for name,size,sha in CURRENT_KERNEL_MODULE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_size_search,actual_before_size_search|size_search_extension)
        for name,size,sha in CURRENT_SIZE_SEARCH_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_recursive_tree,actual_before_recursive_tree|recursive_tree_extension)
        for name,size,sha in CURRENT_RECURSIVE_TREE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_transport,actual_before_transport|transport_extension)
        for name,size,sha in CURRENT_TRANSPORT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_content_backup_lesson,actual_before_content_backup_lesson|content_backup_lesson_extension)
        for name,size,sha in CURRENT_CONTENT_BACKUP_LESSON_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_directory_policy_workspace,actual_before_directory_policy_workspace|directory_policy_workspace_extension)
        for name,size,sha in CURRENT_DIRECTORY_POLICY_WORKSPACE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_directory_policy_core,actual_before_directory_policy_core|directory_policy_core_extension)
        for name,size,sha in CURRENT_DIRECTORY_POLICY_CORE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_whereis_adapter,actual_before_whereis_adapter|whereis_adapter_extension)
        for name,size,sha in CURRENT_WHEREIS_ADAPTER_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_whereis_core,actual_before_whereis_core|whereis_core_extension)
        for name,size,sha in CURRENT_WHEREIS_CORE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_file_index_workspace,actual_before_file_index_workspace|file_index_workspace_extension)
        for name,size,sha in CURRENT_FILE_INDEX_WORKSPACE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_file_index_core,actual_before_file_index_core|file_index_core_extension)
        for name,size,sha in CURRENT_FILE_INDEX_CORE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_command_location_lesson,actual_before_command_location_lesson|command_location_lesson_extension)
        for name,size,sha in CURRENT_COMMAND_LOCATION_LESSON_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_command_location_binding,actual_before_command_location_binding|command_location_binding_extension)
        for name,size,sha in CURRENT_COMMAND_LOCATION_BINDING_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_command_location_core,actual_before_command_location_core|command_location_core_extension)
        for name,size,sha in CURRENT_COMMAND_LOCATION_CORE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_skeleton_adapter,actual_before_skeleton_adapter|skeleton_adapter_extension)
        for name,size,sha in CURRENT_SKELETON_ADAPTER_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_skeleton_core,actual_before_skeleton_core|skeleton_core_extension)
        for name,size,sha in CURRENT_SKELETON_CORE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_path_lookup,actual_before_path_lookup|path_lookup_extension)
        for name,size,sha in CURRENT_PATH_LOOKUP_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_listener,actual_before_listener|listener_extension)
        for name,size,sha in CURRENT_LISTENER_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_managed_config,actual_before_managed_config|managed_config_extension)
        for name,size,sha in CURRENT_MANAGED_CONFIG_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_fhs_content,actual_before_fhs_content|fhs_content_extension)
        for name,size,sha in CURRENT_FHS_CONTENT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_shared_library,actual_before_shared_library|shared_library_extension)
        for name,size,sha in CURRENT_SHARED_LIBRARY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_presentation_paint,actual_before_presentation_paint|presentation_paint_extension)
        for name,size,sha in CURRENT_PRESENTATION_PAINT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_course_metadata,actual_before_course_metadata|course_metadata_extension)
        for name,size,sha in CURRENT_COURSE_METADATA_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_associative_lesson,actual_before_associative_lesson|associative_lesson_extension)
        for name,size,sha in CURRENT_ASSOCIATIVE_LESSON_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_associative,actual_before_associative|associative_extension)
        for name,size,sha in CURRENT_ASSOCIATIVE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_raid1,actual_before_raid1|raid1_extension)
        for name,size,sha in CURRENT_RAID1_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_debian_conffile,actual_before_debian_conffile|debian_conffile_extension)
        for name,size,sha in CURRENT_DEBIAN_CONFFILE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_shell_file_lifecycle_lesson,actual_before_shell_file_lifecycle_lesson|shell_file_lifecycle_lesson_extension)
        for name,size,sha in CURRENT_SHELL_FILE_LIFECYCLE_LESSON_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_shell_file_lifecycle,actual_before_shell_file_lifecycle|shell_file_lifecycle_extension)
        for name,size,sha in CURRENT_SHELL_FILE_LIFECYCLE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_shell_startup,actual_before_shell_startup|shell_startup_extension)
        for name,size,sha in CURRENT_SHELL_STARTUP_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_fs_integrity,actual_before_fs_integrity|fs_integrity_extension)
        for name,size,sha in CURRENT_FS_INTEGRITY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_nm_dhcp,actual_before_nm_dhcp|nm_dhcp_extension)
        for name,size,sha in CURRENT_NM_DHCP_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_debian_conflicts,actual_before_debian_conflicts|debian_conflicts_extension)
        for name,size,sha in CURRENT_DEBIAN_CONFLICT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_user_mounts,actual_before_user_mounts|user_mounts_extension)
        for name,size,sha in CURRENT_USER_MOUNTS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_shell_environment,actual_before_shell_environment|shell_environment_extension)
        for name,size,sha in CURRENT_SHELL_ENVIRONMENT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_shell_tracing,actual_before_shell_tracing|shell_tracing_extension)
        for name,size,sha in CURRENT_SHELL_TRACING_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_debian_source_ui,actual_before_debian_source_ui|debian_source_ui_extension)
        for name,size,sha in CURRENT_DEBIAN_SOURCE_UI_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_debian_source_content,actual_before_debian_source_content|debian_source_content_extension)
        for name,size,sha in CURRENT_DEBIAN_SOURCE_CONTENT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_debian_source_workspace,actual_before_debian_source_workspace|debian_source_workspace_extension)
        for name,size,sha in CURRENT_DEBIAN_SOURCE_WORKSPACE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_debian_source_selection,actual_before_debian_source_selection|debian_source_selection_extension)
        for name,size,sha in CURRENT_DEBIAN_SOURCE_SELECTION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_debian_content_ui,actual_before_debian_content_ui|debian_content_ui_extension)
        for name,size,sha in CURRENT_DEBIAN_CONTENT_UI_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_debian_content,actual_before_debian_content|debian_content_extension)
        for name,size,sha in CURRENT_DEBIAN_CONTENT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_debian_contents,actual_before_debian_contents|debian_contents_extension)
        for name,size,sha in CURRENT_DEBIAN_CONTENTS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_resolver_ui,actual_before_resolver_ui|resolver_ui_extension)
        for name,size,sha in CURRENT_RESOLVER_UI_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_resolver_content,actual_before_resolver_content|resolver_content_extension)
        for name,size,sha in CURRENT_RESOLVER_CONTENT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_resolver_search,actual_before_resolver_search|resolver_search_extension)
        for name,size,sha in CURRENT_RESOLVER_SEARCH_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_client_name_lessons,actual_before_client_name_lessons|client_name_lessons_extension)
        for name,size,sha in CURRENT_CLIENT_NAME_LESSONS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_client_names_workspace,actual_before_client_names_workspace|client_names_workspace_extension)
        for name,size,sha in CURRENT_CLIENT_NAMES_WORKSPACE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_client_names,actual_before_client_names|client_names_extension)
        for name,size,sha in CURRENT_CLIENT_NAMES_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_removable_lesson,actual_before_removable_lesson|removable_lesson_extension)
        for name,size,sha in CURRENT_REMOVABLE_LESSON_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_removable_workspace,actual_before_removable_workspace|removable_workspace_extension)
        for name,size,sha in CURRENT_REMOVABLE_WORKSPACE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_removable_mount,actual_before_removable_mount|removable_mount_extension)
        for name,size,sha in CURRENT_REMOVABLE_MOUNT_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_nm_workspace,actual_before_nm_workspace|nm_workspace_extension)
        for name,size,sha in CURRENT_NM_WORKSPACE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_nm_policy,actual_before_nm_policy|nm_policy_extension)
        for name,size,sha in CURRENT_NM_POLICY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_nm_profile,actual_before_nm_profile|nm_profile_extension)
        for name,size,sha in CURRENT_NM_PROFILE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_dhcp_workspace,actual_before_dhcp_workspace|dhcp_workspace_extension)
        for name,size,sha in CURRENT_DHCP_WORKSPACE_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_dhcp_config,actual_before_dhcp_config|dhcp_config_extension)
        for name,size,sha in CURRENT_DHCP_CONFIG_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_backup_selection,actual_before_backup_selection|backup_selection_extension)
        for name,size,sha in CURRENT_BACKUP_SELECTION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_qos_ui,actual_before_qos_ui|qos_ui_extension)
        for name,size,sha in CURRENT_QOS_UI_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_qos_authored,actual_before_qos_authored|qos_authored_extension)
        for name,size,sha in CURRENT_QOS_AUTHORED_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_qos,actual_before_qos|qos_extension)
        for name,size,sha in CURRENT_QOS_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_startup,actual_before_startup|startup_extension)
        for name,size,sha in CURRENT_STARTUP_LESSONS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_linux_logging,actual_before_linux_logging|linux_logging_extension)
        for name,size,sha in CURRENT_LINUX_LOGGING_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_debug_recovery,actual_before_debug_recovery|debug_recovery_extension)
        for name,size,sha in CURRENT_DEBUG_RECOVERY_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_host,actual_before_host|host_extension)
        for name,size,sha in CURRENT_HOST_FOUNDATION_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_repositories,actual_before_repositories|repository_extension)
        for name,size,sha in CURRENT_REPOSITORY_LESSONS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_capacity,actual_before_capacity|capacity_extension)
        for name,size,sha in CURRENT_CAPACITY_LESSONS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))

        self.assertEqual(current_with_linux,actual_before_linux|linux_extension)
        for name,size,sha in CURRENT_LINUX_FOUNDATIONS_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode)and info.st_nlink==1)
            self.assertEqual((info.st_size,c.digest(path.read_bytes())),(size,sha))



    def test_closed_project_roster_refuses_missing_extra_nested_and_alias_inputs(self):
        import os
        names = ('src/lib.rs', 'scenarios/fixed.json', 'assets/themes/fixed.json')
        for name in names:
            self.put(name, 'SYNTHETIC source, never compiled or executed')
        with patch.object(c, 'COMPILED', names), patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            extra = self.put('src/unreviewed.rs', 'SYNTHETIC extra')
            with self.assertRaises(c.safe.OracleError): c.project_roster(self.root)
            extra.unlink()
            directory = self.root / 'src/unreviewed'; directory.mkdir()
            with self.assertRaises(c.safe.OracleError): c.project_roster(self.root)
            directory.rmdir()
            target = self.root / names[0]; target.unlink()
            with self.assertRaises(c.safe.OracleError): c.project_roster(self.root)
            outside = self.put('outside', 'SYNTHETIC alias')
            target.symlink_to(outside)
            with self.assertRaises(c.safe.OracleError): c.project_roster(self.root)
            target.unlink(); os.link(outside, target)
            with self.assertRaises(c.safe.OracleError): c.project_roster(self.root)
            target.unlink(); self.put(names[0], 'SYNTHETIC restored')
            self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()

    def test_large_retained_snapshot_ignores_current_sources_docs_fixture_and_checks_every_file(self):
        # Reuse the original complete synthetic capture/negative assertions.
        # This creates no real evaluator/GNU observation or executable.
        self.put('src/lib.rs', 'DIFFERENT later current library')
        self.put('docs/SHELL_SUBSET.md', 'DIFFERENT later current docs')
        self.put(c.FIXTURE, 'DIFFERENT later current fixture')
        ordinary_read = c.safe.read_bytes
        def retained_only(path, cap):
            relative = Path(path).relative_to(self.root).as_posix()
            if relative == c.FIXTURE or Path(relative).parts[0] in (*c.SOURCE_FAMILIES, 'docs'):
                raise AssertionError('Verifier reopened later current source/docs/fixture: ' + relative)
            return ordinary_read(path, cap)
        with patch.object(c, 'CORE_PINS', {}), \
             patch.object(c, 'PROBE_SOURCE_SHA', c.digest(b'SYNTHETIC source')), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c, 'project_roster', side_effect=AssertionError('No current-source scan in verify')), \
             patch.object(c.safe, 'read_bytes', side_effect=retained_only), \
             patch.object(c.safe, 'capture_command') as child:
            self.test_readonly_verifier_checks_full_synthetic_retained_capture_and_rejects_tampering()
            out = self.root / c.BASE / c.CAPTURE
            manifest = c.strict((out / 'MANIFEST.json').read_bytes())
            # The old negative finishes accepted=False; restore that SYNTHETIC
            # premise before independent missing/tampered-source controls.
            manifest['accepted'] = True
            raw = c.canonical(manifest); (out / 'MANIFEST.json').write_bytes(raw)
            for name in ('COMPLETE', 'COMPLETION-PREPARED'):
                (out / name).write_bytes((c.digest(raw) + '\n').encode())
            self.assertGreater(len(c.output_paths(self.fixture)), 256)
            self.assertEqual(c.verify(self.root)['status'], 'VERIFIED24-FIXED-UTILITY-COMPARISONS')
            source = out / 'source' / f'file{c.SOURCES.index("src/lib.rs")}.raw'
            original = source.read_bytes(); source.unlink()
            with self.assertRaises((c.safe.OracleError, OSError)): c.verify(self.root)
            source.write_bytes(original + b'SYNTHETIC tampering')
            with self.assertRaises(c.safe.OracleError): c.verify(self.root)
            source.write_bytes(original)
            self.assertEqual(c.verify(self.root)['status'], 'VERIFIED24-FIXED-UTILITY-COMPARISONS')
            child.assert_not_called()


    def test_frozen_r1_prepare_refuses_each_named_current_qa_addition_before_any_child(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_NATIVE_QA_INPUTS:
                path = self.put(name, 'SYNTHETIC named later QA file')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                # Remove only exact newly-created disposable mock paths.
                path.unlink()
                parent = path.parent
                while parent != self.root / 'src':
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()


    def test_frozen_r1_prepare_refuses_each_new_storage_product_input_before_child_or_control(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_STORAGE_WORKSPACE_INPUTS:
                path = self.put(name, 'SYNTHETIC named later storage product file')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                # Remove only exact newly-created disposable mock paths.
                path.unlink()
                parent = path.parent
                while parent != self.root / 'src':
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()



    def test_frozen_r1_prepare_refuses_new_literal_product_before_child_or_control(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_LITERAL_PRODUCT_INPUTS:
                path = self.put(name, 'SYNTHETIC named later literal product file')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                # Remove only exact newly-created disposable mock paths.
                path.unlink()
                parent = path.parent
                while parent != self.root / 'src':
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()



    def test_frozen_r1_prepare_refuses_unregistered_storage_pack_before_child_or_control(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_AUTHORED_STORAGE_INPUTS:
                path = self.put(name, 'SYNTHETIC unregistered storage pack')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                path.unlink()  # Only this exact new disposable mock leaf; family already existed.
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()


    def test_frozen_r1_prepare_refuses_conditional_storage_native_modules_before_children(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_STORAGE_NATIVE_INPUTS:
                path = self.put(name, 'SYNTHETIC later storage native QA file')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                path.unlink()
                parent = path.parent
                while parent != self.root / 'src':
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()

    def test_frozen_r1_prepare_refuses_literal_content_pack_before_control_destination_or_subject(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        for name, size, sha in CURRENT_LITERAL_AUTHORED_INPUTS:
            raw = (PROJECT / name).read_bytes()
            self.assertEqual(len(raw), size)
            self.assertEqual(c.digest(raw), sha)
            added = self.put(name, raw)
            with patch.object(c, 'COMPILED', names), \
                 patch.object(c.shared, 'root_guard', return_value=()), \
                 patch.object(c, 'fixed', return_value=self.fixture), \
                 patch.object(c, 'control') as control, \
                 patch.object(c.shared, 'owned_directory') as destination, \
                 patch.object(c.shared, 'pinned_executable') as held, \
                 patch.object(c.safe, 'capture_command') as child:
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                control.assert_not_called()
                destination.assert_not_called()
                held.assert_not_called()
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                # Only the exact newly-created private mock leaf is removed.
                added.unlink()
                self.assertEqual(c.project_roster(self.root), set(names))


    def test_frozen_r1_prepare_refuses_each_hsrp_workspace_source_before_child_or_control(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_HSRP_WORKSPACE_INPUTS:
                path = self.put(name, 'SYNTHETIC unobserved HSRP workspace source')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                path.unlink()  # Only this exact new disposable mock leaf.
                parent = path.parent
                while parent != self.root / 'src':
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()


    def test_frozen_r1_prepare_refuses_unregistered_hsrp_pack_before_child_or_control(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_AUTHORED_HSRP_INPUTS:
                path = self.put(name, 'SYNTHETIC unregistered HSRP authored pack')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                path.unlink()  # Only this exact new disposable leaf; family existed.
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()

    def test_frozen_r1_prepare_refuses_both_literal_native_modules_before_subject_or_destination(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        for name in names:self.put(name,'SYNTHETIC closed input')
        for name,size,sha in CURRENT_LITERAL_NATIVE_INPUTS:
            raw=(PROJECT/name).read_bytes()
            self.assertEqual(len(raw),size);self.assertEqual(c.digest(raw),sha)
            added=self.put(name,raw)
            with patch.object(c,'COMPILED',names), patch.object(c.shared,'root_guard',return_value=()), patch.object(c,'fixed',return_value=self.fixture), patch.object(c,'control') as control, patch.object(c.shared,'owned_directory') as destination, patch.object(c.shared,'pinned_executable') as held, patch.object(c.safe,'capture_command') as child:
                with self.assertRaises(c.safe.OracleError) as refused:c.run(self.root,'prepare')
                self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                self.assertFalse((self.root/c.BASE).exists())
                added.unlink()
                parent=added.parent
                while parent!=self.root/'src':
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_r1_prepare_refuses_both_snapshot_native_modules_before_subject_or_destination(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        for name in names:self.put(name,'SYNTHETIC closed input')
        for name,size,sha in CURRENT_SNAPSHOT_NATIVE_INPUTS:
            raw=(PROJECT/name).read_bytes()
            self.assertEqual(len(raw),size);self.assertEqual(c.digest(raw),sha)
            added=self.put(name,raw)
            with patch.object(c,'COMPILED',names), patch.object(c.shared,'root_guard',return_value=()), patch.object(c,'fixed',return_value=self.fixture), patch.object(c,'control') as control, patch.object(c.shared,'owned_directory') as destination, patch.object(c.shared,'pinned_executable') as held, patch.object(c.safe,'capture_command') as child:
                with self.assertRaises(c.safe.OracleError) as refused:c.run(self.root,'prepare')
                self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                self.assertFalse((self.root/c.BASE).exists())
                added.unlink()
                parent=added.parent
                while parent!=self.root/'src':
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_prepare_refuses_each_syslog_source_before_control_or_subject(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        with patch.object(c, "COMPILED", names):
            for name in names:
                self.put(name, 'SYNTHETIC closed original input')
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, size, sha in CURRENT_SYSLOG_WORKSPACE_INPUTS:
                source = (PROJECT / name).read_bytes()
                self.assertEqual((len(source), c.digest(source)), (size, sha))
                added = self.put(name, source)
                with patch.object(c, 'COMPILED', names), \
                     patch.object(c.shared, 'root_guard', return_value=()), \
                     patch.object(c, 'fixed', return_value=self.fixture), \
                     patch.object(c, 'control') as control, \
                     patch.object(c.shared, 'owned_directory') as destination, \
                     patch.object(c.shared, 'pinned_executable') as held, \
                     patch.object(c.safe, 'capture_command') as child:
                    with self.assertRaises(c.safe.OracleError) as refused:
                        c.run(self.root, 'prepare')
                    self.assertRegex(str(refused.exception),
                                     'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called()
                    destination.assert_not_called()
                    held.assert_not_called()
                    child.assert_not_called()
                    self.assertFalse((self.root / c.BASE).exists())
                added.unlink()  # Only the exact newly-created disposable mock leaf.
                parent = added.parent
                family = self.root / Path(name).parts[0]
                while parent != family:
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))

    def test_frozen_reference_prepare_refuses_unregistered_syslog_pack_before_control_or_subject(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        with patch.object(c, "COMPILED", names):
            for name in names:
                self.put(name, 'SYNTHETIC closed original input')
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, size, sha in CURRENT_AUTHORED_SYSLOG_INPUTS:
                source = (PROJECT / name).read_bytes()
                self.assertEqual((len(source), c.digest(source)), (size, sha))
                added = self.put(name, source)
                with patch.object(c, 'COMPILED', names), \
                     patch.object(c.shared, 'root_guard', return_value=()), \
                     patch.object(c, 'fixed', return_value=self.fixture), \
                     patch.object(c, 'control') as control, \
                     patch.object(c.shared, 'owned_directory') as destination, \
                     patch.object(c.shared, 'pinned_executable') as held, \
                     patch.object(c.safe, 'capture_command') as child:
                    with self.assertRaises(c.safe.OracleError) as refused:
                        c.run(self.root, 'prepare')
                    self.assertRegex(str(refused.exception),
                                     'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called()
                    destination.assert_not_called()
                    held.assert_not_called()
                    child.assert_not_called()
                    self.assertFalse((self.root / c.BASE).exists())
                added.unlink()  # Only the exact newly-created disposable mock leaf.
                parent = added.parent
                family = self.root / Path(name).parts[0]
                while parent != family:
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))

    def test_frozen_reference_refuses_every_current_teaching_and_links_source_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_TEACHING_LINKS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()), patch.object(c,'fixed',return_value=self.fixture), patch.object(c,'control') as control, patch.object(c.shared,'owned_directory') as destination, patch.object(c.shared,'pinned_executable') as held, patch.object(c.safe,'capture_command') as child:
                    with self.assertRaises(c.safe.OracleError) as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact disposable mock leaf only; retained evidence is untouched.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_each_pure_linux_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LINUX_FOUNDATIONS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_each_capacity_lesson_input_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_CAPACITY_LESSONS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_each_repository_lesson_input_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_REPOSITORY_LESSONS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_host_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_HOST_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))






    def test_frozen_reference_refuses_debug_recovery_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEBUG_RECOVERY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_linux_logging_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LINUX_LOGGING_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_startup_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_STARTUP_LESSONS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_qos_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_QOS_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_qos_authored_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_QOS_AUTHORED_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_qos_ui_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_QOS_UI_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_backup_selection_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_BACKUP_SELECTION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_dhcp_config_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DHCP_CONFIG_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_dhcp_workspace_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DHCP_WORKSPACE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_nm_profile_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_NM_PROFILE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_nm_policy_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_NM_POLICY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_nm_workspace_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_NM_WORKSPACE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_removable_mount_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_REMOVABLE_MOUNT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_removable_workspace_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_REMOVABLE_WORKSPACE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_removable_lesson_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_REMOVABLE_LESSON_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_client_names_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_CLIENT_NAMES_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_client_names_workspace_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_CLIENT_NAMES_WORKSPACE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_client_name_lessons_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_CLIENT_NAME_LESSONS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_resolver_search_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RESOLVER_SEARCH_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_resolver_content_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RESOLVER_CONTENT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_resolver_ui_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RESOLVER_UI_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_debian_contents_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEBIAN_CONTENTS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_debian_content_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEBIAN_CONTENT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_debian_content_ui_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEBIAN_CONTENT_UI_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_debian_source_selection_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEBIAN_SOURCE_SELECTION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_debian_source_workspace_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEBIAN_SOURCE_WORKSPACE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_debian_source_content_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEBIAN_SOURCE_CONTENT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_debian_source_ui_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEBIAN_SOURCE_UI_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_shell_tracing_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SHELL_TRACING_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_shell_environment_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SHELL_ENVIRONMENT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_user_mounts_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_USER_MOUNTS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_debian_conflicts_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEBIAN_CONFLICT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_nm_dhcp_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_NM_DHCP_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_fs_integrity_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_FS_INTEGRITY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_shell_startup_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SHELL_STARTUP_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_shell_file_lifecycle_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SHELL_FILE_LIFECYCLE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_shell_file_lifecycle_lesson_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SHELL_FILE_LIFECYCLE_LESSON_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_debian_conffile_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEBIAN_CONFFILE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_raid1_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RAID1_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_associative_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_ASSOCIATIVE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_associative_lesson_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_ASSOCIATIVE_LESSON_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_course_metadata_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_COURSE_METADATA_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_presentation_paint_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PRESENTATION_PAINT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_shared_library_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SHARED_LIBRARY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_fhs_content_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_FHS_CONTENT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_managed_config_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_MANAGED_CONFIG_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_listener_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LISTENER_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_path_lookup_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PATH_LOOKUP_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_skeleton_core_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SKELETON_CORE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_skeleton_adapter_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SKELETON_ADAPTER_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_command_location_core_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_COMMAND_LOCATION_CORE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_command_location_binding_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_COMMAND_LOCATION_BINDING_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_command_location_lesson_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_COMMAND_LOCATION_LESSON_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_file_index_core_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_FILE_INDEX_CORE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_file_index_workspace_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_FILE_INDEX_WORKSPACE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_whereis_core_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_WHEREIS_CORE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_whereis_adapter_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_WHEREIS_ADAPTER_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_directory_policy_core_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DIRECTORY_POLICY_CORE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_directory_policy_workspace_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DIRECTORY_POLICY_WORKSPACE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_content_backup_lesson_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_CONTENT_BACKUP_LESSON_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_transport_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_TRANSPORT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_recursive_tree_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RECURSIVE_TREE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_size_search_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SIZE_SEARCH_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_kernel_module_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_KERNEL_MODULE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_kernel_names_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_KERNEL_NAMES_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_log_triage_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LOG_TRIAGE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_rpm_metadata_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RPM_METADATA_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_host_ipv6_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_HOST_IPV6_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_swap_policy_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SWAP_POLICY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_lvm_provisioning_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LVM_PROVISIONING_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_lvm_retirement_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LVM_RETIREMENT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_lvm_activation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LVM_ACTIVATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_lvm_rename_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LVM_RENAME_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_pv_capacity_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PV_CAPACITY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_mbr_layout_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_MBR_LAYOUT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_cow_model_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_COW_MODEL_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_device_path_model_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DEVICE_PATH_MODEL_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_lvm_config_scan_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LVM_CONFIG_SCAN_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_lvm_mount_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LVM_MOUNT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_autofs_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_AUTOFS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_raid_assembly_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RAID_ASSEMBLY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_vlan_host_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_VLAN_HOST_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_bond_control_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_BOND_CONTROL_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_bridge_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_BRIDGE_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_pam_control_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PAM_CONTROL_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_aging_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_AGING_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_static_cam_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_STATIC_CAM_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_port_security_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PORT_SECURITY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_access_outputs_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_ACCESS_OUTPUTS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_dhcp_snooping_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DHCP_SNOOPING_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_arp_inspection_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_ARP_INSPECTION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_management_policy_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_MANAGEMENT_POLICY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_local_access_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LOCAL_ACCESS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_aaa_methods_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_AAA_METHODS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_ldap_query_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LDAP_QUERY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_ldap_changes_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LDAP_CHANGE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_ldap_utility_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LDAP_UTILITY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_ldap_delete_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LDAP_DELETE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_ldap_protection_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LDAP_PROTECTION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_ldap_access_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LDAP_ACCESS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_ldap_configuration_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LDAP_CONFIGURATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_ldap_schema_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LDAP_SCHEMA_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_eui64_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_EUI64_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_boot_entry_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_BOOT_ENTRY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_grub_menu_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_GRUB_MENU_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_grub_settings_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_GRUB_SETTINGS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_bios_loader_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_BIOS_LOADER_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_efi_loader_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_EFI_LOADER_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_efi_removable_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_EFI_REMOVABLE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_grub_once_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_GRUB_ONCE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_rescue_startup_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RESCUE_STARTUP_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_chrony_source_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_CHRONY_SOURCE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_journal_archive_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_COPIED_JOURNAL_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_copied_journal_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_COPIED_JOURNAL_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_local_mail_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LOCAL_MAIL_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_wired_device_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_WIRED_DEVICE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_json_inventory_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_JSON_INVENTORY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_layout_policy_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LAYOUT_POLICY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_resource_measurement_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RESOURCE_MEASUREMENT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_resource_observation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RESOURCE_OBSERVATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_capacity_forecast_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_CAPACITY_FORECAST_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_maintenance_notice_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_MAINTENANCE_NOTICE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_source_install_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SOURCE_INSTALL_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_dns_access_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DNS_ACCESS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_print_queues_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PRINT_QUEUES_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_localization_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LOCALIZATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_timezones_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_TIMEZONES_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_wireless_channels_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_WIRELESS_CHANNEL_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_wlan_identity_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_WLAN_IDENTITY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_wlan_paths_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_WLAN_PATH_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_snmp_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SNMP_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_proxy_policy_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PROXY_POLICY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_http_cache_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_HTTP_CACHE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_mixed_ip_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_MIXED_IP_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_host_echo_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_HOST_ECHO_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_rest_controller_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_REST_CONTROLLER_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_client_diagnostic_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_CLIENT_DIAGNOSTIC_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))



    def test_frozen_reference_refuses_campus_architecture_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_CAMPUS_ARCHITECTURE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))








    def test_frozen_reference_refuses_leaf_spine_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LEAF_SPINE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))








    def test_frozen_reference_refuses_mailbox_admin_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_MAILBOX_ADMIN_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_linux_router_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LINUX_ROUTER_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_linux_connections_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LINUX_CONNECTION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_linux_ipv6_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LINUX_IPV6_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_linux_service_rules_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LINUX_SERVICE_RULE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_linux_v6_admin_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LINUX_V6_ADMIN_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_linux_v6_startup_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LINUX_V6_STARTUP_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_rpm_configuration_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_RPM_CONFIGURATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_ids_rule_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_IDS_RULE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_nc_probe_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_NC_PROBE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_ospf_broadcast_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_OSPF_BROADCAST_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_route_reading_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_ROUTE_READING_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_nfs_export_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_NFS_EXPORT_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_vlan_gateway_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_VLAN_GATEWAY_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_host_dns_security_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_HOST_DNS_SECURITY_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_site_connectivity_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SITE_CONNECTIVITY_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_network_evidence_review_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_NETWORK_EVIDENCE_REVIEW_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_staged_network_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_STAGED_NETWORK_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_credentials_vpn_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_CREDENTIALS_VPN_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_security_investigation_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SECURITY_INVESTIGATION_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_desktop_profile_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DESKTOP_PROFILE_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_apache_https_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_APACHE_HTTPS_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))
    def test_frozen_reference_refuses_ata_smart_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_ATA_SMART_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_iscsi_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_ISCSI_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_pxe_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PXE_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))



    def test_frozen_reference_refuses_initramfs_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_INITRAMFS_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))




    def test_frozen_reference_refuses_dkms_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DKMS_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))




    def test_frozen_reference_refuses_kernel_build_foundation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_KERNEL_BUILD_FOUNDATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))




    def test_frozen_reference_refuses_poe_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_POE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))




    def test_frozen_reference_refuses_telnet_transport_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_TELNET_TRANSPORT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))




    def test_frozen_reference_refuses_log_filter_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_LOG_FILTER_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_vpn_point_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_VPN_POINT_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_service_exposure_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SERVICE_EXPOSURE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

    def test_frozen_reference_refuses_ftp_identity_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_FTP_IDENTITY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

























    def test_frozen_reference_refuses_mailbox_access_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_MAILBOX_ACCESS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

















    def test_frozen_reference_refuses_procmail_migration_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PROCMAIL_MIGRATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))
















    def test_frozen_reference_refuses_sieve_vacation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SIEVE_VACATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))















    def test_frozen_reference_refuses_sieve_delivery_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SIEVE_DELIVERY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))














    def test_frozen_reference_refuses_sieve_address_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SIEVE_ADDRESS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))













    def test_frozen_reference_refuses_sieve_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SIEVE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))












    def test_frozen_reference_refuses_smtp_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SMTP_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))











    def test_frozen_reference_refuses_guest_image_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_GUEST_IMAGE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))










    def test_frozen_reference_refuses_virtual_isolation_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_VIRTUAL_ISOLATION_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))









    def test_frozen_reference_refuses_samba_share_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SAMBA_SHARE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))








    def test_frozen_reference_refuses_network_transfer_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_NETWORK_TRANSFER_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))







    def test_frozen_reference_refuses_handover_replay_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_HANDOVER_REPLAY_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))






    def test_frozen_reference_refuses_editor_handover_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_EDITOR_HANDOVER_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))





    def test_frozen_reference_refuses_modal_editing_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_MODAL_EDITING_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))




    def test_frozen_reference_refuses_ssh_access_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_SSH_ACCESS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))



    def test_frozen_reference_refuses_dns_query_tools_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DNS_QUERY_TOOLS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_protocol_evidence_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PROTOCOL_EVIDENCE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_host_lessons_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_HOST_LESSONS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_direct_dns_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_DIRECT_DNS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_protocol_evidence_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_PROTOCOL_EVIDENCE_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))


    def test_frozen_reference_refuses_host_lessons_before_any_launch(self):
        names=('src/lib.rs','scenarios/base.json','assets/theme.txt')
        with patch.object(c,'COMPILED',names):
            for name in names:self.put(name,'SYNTHETIC closed input')
            self.assertEqual(c.project_roster(self.root),set(names))
            for name,size,sha in CURRENT_HOST_LESSONS_INPUTS:
                raw=(PROJECT/name).read_bytes();self.assertEqual((len(raw),c.digest(raw)),(size,sha))
                added=self.put(name,raw)
                with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'control')as control,patch.object(c.shared,'owned_directory')as destination,patch.object(c.shared,'pinned_executable')as held,patch.object(c.safe,'capture_command')as child:
                    with self.assertRaises(c.safe.OracleError)as refused:c.run(self.root,'prepare')
                    self.assertRegex(str(refused.exception),'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                    control.assert_not_called();destination.assert_not_called();held.assert_not_called();child.assert_not_called()
                    self.assertFalse((self.root/c.BASE).exists())
                added.unlink()  # Exact newly-created disposable leaf, never retained evidence.
                parent=added.parent;family=self.root/Path(name).parts[0]
                while parent!=family:
                    previous=parent;parent=parent.parent;previous.rmdir()
                self.assertEqual(c.project_roster(self.root),set(names))

