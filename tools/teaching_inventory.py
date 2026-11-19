#!/usr/bin/env python3
"""Inventory the registered teaching rollout; no scripts or simulation are executed."""
import argparse
import csv
import io
import json
from collections import Counter
from pathlib import Path
from verify_coverage import registered_pack_paths

ROOT = Path(__file__).resolve().parents[1]
TRACKS = ['CCNA', 'LPIC-1', 'LPIC-2', 'Bash/Shell']
SHARDS = ['ccna', 'lpic1', 'lpic2', 'bash']

def catalog():
    # Resolve the selected registry, rather than counting every include token in
    # historical checkpoint helpers. Keep the strict resolver's cycle/depth/path
    # guards and still reject duplicate lab IDs across distinct selected packs.
    paths = registered_pack_paths(ROOT, 'current')
    labs = []
    for path in paths:
        for lab in json.loads((ROOT / path).read_text())['scenarios']:
            labs.append((lab, path))
    if len({lab['id'] for lab, _ in labs}) != len(labs):
        raise ValueError('Duplicate registered lab ID')
    return labs

def same_preserved_origin(lab, origin, bundled_path):
    if bundled_path == 'scenarios/curriculum-continuation-v19.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v18.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/ccna-ospf-broadcast-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 39 or len(added) != 1 or combined != before + added:
            raise ValueError('broadcast designated roles continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/ccna-ospf-broadcast-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v18.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v18.json')
    if bundled_path == 'scenarios/curriculum-continuation-v18.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v17.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/ccna-route-reading-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 38 or len(added) != 1 or combined != before + added:
            raise ValueError('installed route reading continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/ccna-route-reading-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v17.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v17.json')
    if bundled_path == 'scenarios/curriculum-continuation-v17.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v16.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/nfs-export-operations-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 37 or len(added) != 1 or combined != before + added:
            raise ValueError('NFS export operations continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/nfs-export-operations-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v16.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v16.json')
    if bundled_path == 'scenarios/curriculum-continuation-v16.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v15.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/ccna-vlan-gateway-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 35 or len(added) != 2 or combined != before + added:
            raise ValueError('VLAN gateway repair continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/ccna-vlan-gateway-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v15.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v15.json')
    if bundled_path == 'scenarios/curriculum-continuation-v15.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v14.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/host-and-dns-security-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 33 or len(added) != 2 or combined != before + added:
            raise ValueError('Host and DNS security continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/host-and-dns-security-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v14.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v14.json')
    if bundled_path == 'scenarios/curriculum-continuation-v14.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v13.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/site-connectivity-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 31 or len(added) != 2 or combined != before + added:
            raise ValueError('Site connectivity continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/site-connectivity-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v13.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v13.json')
    if bundled_path == 'scenarios/curriculum-continuation-v13.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v12.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/network-evidence-review-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 29 or len(added) != 2 or combined != before + added:
            raise ValueError('Network evidence review continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/network-evidence-review-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v12.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v12.json')
    if bundled_path == 'scenarios/curriculum-continuation-v12.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v11.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/staged-network-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 27 or len(added) != 2 or combined != before + added:
            raise ValueError('Staged network changes continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/staged-network-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v11.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v11.json')
    if bundled_path == 'scenarios/curriculum-continuation-v11.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v10.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/credentials-vpn-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 25 or len(added) != 2 or combined != before + added:
            raise ValueError('Credentials and VPN continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/credentials-vpn-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v10.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v10.json')
    if bundled_path == 'scenarios/curriculum-continuation-v10.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v9.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/security-investigation-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 23 or len(added) != 2 or combined != before + added:
            raise ValueError('Security investigation continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/security-investigation-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v9.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v9.json')
    if bundled_path == 'scenarios/curriculum-continuation-v9.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v8.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/desktop-profile-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 20 or len(added) != 3 or combined != before + added:
            raise ValueError('Desktop profile continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/desktop-profile-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v8.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v8.json')
    if bundled_path == 'scenarios/curriculum-continuation-v8.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v7.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/apache-https-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 18 or len(added) != 2 or combined != before + added:
            raise ValueError('Apache HTTPS continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/apache-https-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v7.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v7.json')
    if bundled_path == 'scenarios/curriculum-continuation-v7.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v6.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/ata-smart-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 16 or len(added) != 2 or combined != before + added:
            raise ValueError('ATA SMART continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/ata-smart-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v6.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v6.json')
    if bundled_path == 'scenarios/curriculum-continuation-v6.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v5.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/iscsi-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 14 or len(added) != 2 or combined != before + added:
            raise ValueError('iSCSI continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/iscsi-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v5.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v5.json')
    if bundled_path == 'scenarios/curriculum-continuation-v5.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v4.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/pxe-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 12 or len(added) != 2 or combined != before + added:
            raise ValueError('Network boot continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/pxe-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v4.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v4.json')
    if bundled_path == 'scenarios/curriculum-continuation-v4.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v3.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/initramfs-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 10 or len(added) != 2 or combined != before + added:
            raise ValueError('Early-userspace continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/initramfs-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v3.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v3.json')
    if bundled_path == 'scenarios/curriculum-continuation-v3.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v2.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/dkms-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 8 or len(added) != 2 or combined != before + added:
            raise ValueError('DKMS continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/dkms-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v2.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v2.json')
    if bundled_path == 'scenarios/curriculum-continuation-v2.json':
        before = json.loads((ROOT/'scenarios/curriculum-continuation-v1.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/kernel-build-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before) != 6 or len(added) != 2 or combined != before + added:
            raise ValueError('Kernel continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/kernel-build-lessons-v1.json':
            return any(old == lab for old in added)
        if origin == 'scenarios/curriculum-continuation-v1.json':
            return any(old == lab for old in before)
        return same_preserved_origin(lab, origin, 'scenarios/curriculum-continuation-v1.json')
    if bundled_path == 'scenarios/curriculum-continuation-v1.json':
        before = json.loads((ROOT/'scenarios/security-detection-lessons-v2.json').read_text())['scenarios']
        added = json.loads((ROOT/'scenarios/poe-lessons-v1.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if combined != before + added:
            raise ValueError('Power continuation aggregate changed exact prior records or additions')
        if origin == 'scenarios/poe-lessons-v1.json':
            return any(old == lab for old in added)
        return same_preserved_origin(lab, origin, 'scenarios/security-detection-lessons-v2.json')
    if bundled_path == 'scenarios/security-detection-lessons-v2.json':
        before=json.loads((ROOT/'scenarios/auth-filter-lessons-v1.json').read_text())['scenarios']
        added=json.loads((ROOT/'scenarios/ids-rule-lessons-v1.json').read_text())['scenarios']
        combined=json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(before)!=2 or len(added)!=2 or combined!=before+added:
            raise ValueError('Security detection consolidation changed exact originals or additions')
        if origin=='scenarios/auth-filter-lessons-v1.json': return any(old==lab for old in before)
        if origin=='scenarios/ids-rule-lessons-v1.json': return any(old==lab for old in added)
        return False
    if bundled_path == 'scenarios/ldap-administration-v3.json':
        client = json.loads((ROOT/'scenarios/ldap-client-lessons-v1.json').read_text())['scenarios']
        administration = json.loads((ROOT/'scenarios/ldap-administration-v2.json').read_text())['scenarios']
        combined = json.loads((ROOT/bundled_path).read_text())['scenarios']
        if len(client) != 2 or len(administration) != 10 or combined != client + administration:
            raise ValueError('LDAP client/admin consolidation changed exact predecessors')
        if origin == 'scenarios/ldap-client-lessons-v1.json':
            return any(old == lab for old in client)
        if origin == 'scenarios/ldap-administration-v2.json':
            return any(old == lab for old in administration)
        return same_preserved_origin(lab, origin, 'scenarios/ldap-administration-v2.json')
    if bundled_path == 'scenarios/boot-entry-lessons-v63.json':
        before=json.loads((ROOT/'scenarios/boot-entry-lessons-v62.json').read_text())["scenarios"]
        added=json.loads((ROOT/'scenarios/advisory-patch-lessons-v1.json').read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Advisory patch consolidation changed exact predecessor or additions")
        if origin=='scenarios/advisory-patch-lessons-v1.json':
            return any(old==lab for old in added)
        if origin=='scenarios/boot-entry-lessons-v62.json':
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,'scenarios/boot-entry-lessons-v62.json')
    if bundled_path == "scenarios/boot-entry-lessons-v62.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v61.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/vpn-point-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("VPN consolidation changed exact predecessor or authored additions")
        if origin=="scenarios/vpn-point-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v61.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v61.json")
    if bundled_path == "scenarios/boot-entry-lessons-v61.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v60.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/service-exposure-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Exposure consolidation changed exact predecessor or authored additions")
        if origin=="scenarios/service-exposure-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v60.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v60.json")
    if bundled_path == "scenarios/boot-entry-lessons-v60.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v59.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/ftp-identity-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("FTP identity consolidation changed exact predecessor or additions")
        if origin=="scenarios/ftp-identity-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v59.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v59.json")
    if bundled_path == "scenarios/boot-entry-lessons-v59.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v58.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/linux-v6-startup-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("IPv6 startup consolidation changed exact predecessor or addition")
        if origin=="scenarios/linux-v6-startup-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v58.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v58.json")
    if bundled_path == "scenarios/boot-entry-lessons-v58.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v57.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/linux-v6-admin-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("IPv6 administration consolidation changed exact predecessor or additions")
        if origin=="scenarios/linux-v6-admin-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v57.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v57.json")
    if bundled_path == "scenarios/boot-entry-lessons-v57.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v56.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/linux-services-router-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Services/router consolidation changed exact predecessor or addition")
        if origin=="scenarios/linux-services-router-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v56.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v56.json")
    if bundled_path == "scenarios/boot-entry-lessons-v56.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v55.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/linux-v6-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("IPv6-router consolidation changed exact predecessor or additions")
        if origin=="scenarios/linux-v6-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v55.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v55.json")
    if bundled_path == "scenarios/boot-entry-lessons-v55.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v54.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/linux-flow-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Linux-flow consolidation changed exact predecessor or additions")
        if origin=="scenarios/linux-flow-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v54.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v54.json")
    if bundled_path == "scenarios/boot-entry-lessons-v54.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v53.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/linux-router-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Linux-router consolidation changed exact predecessor or additions")
        if origin=="scenarios/linux-router-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v53.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v53.json")
    if bundled_path == "scenarios/boot-entry-lessons-v53.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v52.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/mailbox-administration-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Mail-administration consolidation changed exact predecessor or additions")
        if origin=="scenarios/mailbox-administration-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v52.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v52.json")
    if bundled_path == "scenarios/boot-entry-lessons-v52.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v51.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/mailbox-access-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Mailbox-access consolidation changed exact predecessor or additions")
        if origin=="scenarios/mailbox-access-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v51.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v51.json")
    if bundled_path == "scenarios/boot-entry-lessons-v51.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v50.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/procmail-migration-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Legacy-migration consolidation changed exact predecessor or additions")
        if origin=="scenarios/procmail-migration-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v50.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v50.json")
    if bundled_path == "scenarios/boot-entry-lessons-v50.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v49.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/sieve-vacation-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Vacation-filter consolidation changed exact predecessor or additions")
        if origin=="scenarios/sieve-vacation-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v49.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v49.json")
    if bundled_path == "scenarios/boot-entry-lessons-v49.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v48.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/sieve-delivery-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Delivery-filter consolidation changed exact predecessor or additions")
        if origin=="scenarios/sieve-delivery-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v48.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v48.json")
    if bundled_path == "scenarios/boot-entry-lessons-v48.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v47.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/sieve-address-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Address-filter consolidation changed exact predecessor or additions")
        if origin=="scenarios/sieve-address-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v47.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v47.json")
    if bundled_path == "scenarios/boot-entry-lessons-v47.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v46.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/sieve-filter-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Sieve consolidation changed exact predecessor or additions")
        if origin=="scenarios/sieve-filter-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v46.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v46.json")
    if bundled_path == "scenarios/boot-entry-lessons-v46.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v45.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/smtp-routing-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("SMTP consolidation changed exact predecessor or additions")
        if origin=="scenarios/smtp-routing-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v45.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v45.json")
    if bundled_path == "scenarios/boot-entry-lessons-v45.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v44.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/guest-driver-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Guest-image consolidation changed exact predecessor or additions")
        if origin=="scenarios/guest-driver-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v44.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v44.json")
    if bundled_path == "scenarios/boot-entry-lessons-v44.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v43.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/virtual-isolation-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Virtual-isolation consolidation changed exact predecessor or additions")
        if origin=="scenarios/virtual-isolation-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v43.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v43.json")
    if bundled_path == "scenarios/boot-entry-lessons-v43.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v42.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/samba-standalone-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Standalone-sharing consolidation changed exact predecessor or additions")
        if origin=="scenarios/samba-standalone-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v42.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v42.json")
    if bundled_path == "scenarios/boot-entry-lessons-v42.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v41.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/leaf-spine-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Leaf/spine consolidation changed exact predecessor or authored addition")
        if origin=="scenarios/leaf-spine-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v41.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v41.json")
    if bundled_path == "scenarios/boot-entry-lessons-v41.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v40.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/campus-architecture-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Campus architecture consolidation changed exact predecessor or authored additions")
        if origin=="scenarios/campus-architecture-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v40.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v40.json")
    if bundled_path == "scenarios/boot-entry-lessons-v40.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v39.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/network-transfers-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Network-transfer consolidation changed exact predecessor or authored additions")
        if origin=="scenarios/network-transfers-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v39.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v39.json")
    if bundled_path == "scenarios/boot-entry-lessons-v39.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v38.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/editor-handover-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Editor-handover consolidation changed the exact predecessor or authored addition")
        if origin=="scenarios/editor-handover-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v38.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v38.json")
    if bundled_path == "scenarios/boot-entry-lessons-v38.json":
        before=json.loads((ROOT/"scenarios/boot-entry-lessons-v37.json").read_text())["scenarios"]
        added=json.loads((ROOT/"scenarios/modal-editing-lessons-v1.json").read_text())["scenarios"]
        combined=json.loads((ROOT/bundled_path).read_text())["scenarios"]
        if combined != before+added:
            raise ValueError("Modal consolidation changed the exact predecessor or authored additions")
        if origin=="scenarios/modal-editing-lessons-v1.json":
            return any(old==lab for old in added)
        if origin=="scenarios/boot-entry-lessons-v37.json":
            return any(old==lab for old in before)
        return any(old==lab for old in before) and same_preserved_origin(lab,origin,"scenarios/boot-entry-lessons-v37.json")
    if bundled_path == "scenarios/boot-entry-lessons-v37.json" and origin in {"scenarios/boot-entry-lessons-v36.json", "scenarios/ssh-access-lessons-v1.json"}:
        original = json.loads((ROOT / origin).read_text())["scenarios"]
        return any(old == lab for old in original)
    if origin == 'scenarios/dns-forward-v1.json' and bundled_path == 'scenarios/dns-forward-v2.json':
        original = json.loads((ROOT / origin).read_text())['scenarios']
        return any(old == lab for old in original)
    if origin == 'scenarios/linux-host-lessons-v1.json' and bundled_path == 'scenarios/linux-host-lessons-v2.json':
        original = json.loads((ROOT / origin).read_text())['scenarios']
        return any(old == lab for old in original)
    # Two adjacent source-owned LDAP packs are consolidated without changing
    # scenario values or order. Keep the actual authored origins in old reviews;
    # accept only a complete identical scenario, never an arbitrary path alias.
    expected = {'scenarios/ldap-change-lessons-v1.json': 2,
                'scenarios/ldap-utility-lessons-v1.json': 8,
                'scenarios/kernel-module-lessons-v1.json': 3,
                'scenarios/kernel-module-names-lessons-v1.json': 1,
                'scenarios/boot-entry-lessons-v1.json': 3,
                'scenarios/boot-entry-lessons-v2.json': 5,
                'scenarios/boot-entry-lessons-v3.json': 7,
                'scenarios/boot-entry-lessons-v4.json': 8,
                'scenarios/boot-entry-lessons-v5.json': 9,
                'scenarios/boot-entry-lessons-v6.json': 10,
                'scenarios/boot-entry-lessons-v7.json': 11,
                'scenarios/boot-entry-lessons-v8.json': 12,
                'scenarios/boot-entry-lessons-v9.json': 13,
                'scenarios/boot-entry-lessons-v10.json': 14,
                'scenarios/boot-entry-lessons-v11.json': 17,
                'scenarios/boot-entry-lessons-v12.json': 19,
                'scenarios/boot-entry-lessons-v13.json': 21,
                'scenarios/boot-entry-lessons-v14.json': 23,
                'scenarios/boot-entry-lessons-v15.json': 26,
                'scenarios/boot-entry-lessons-v16.json': 29,
                'scenarios/boot-entry-lessons-v17.json': 31,
                'scenarios/boot-entry-lessons-v18.json': 33,
                'scenarios/boot-entry-lessons-v19.json': 36,
                'scenarios/boot-entry-lessons-v20.json': 39,
                'scenarios/boot-entry-lessons-v21.json': 42,
                'scenarios/boot-entry-lessons-v22.json': 44,
                'scenarios/boot-entry-lessons-v23.json': 46,
                'scenarios/boot-entry-lessons-v24.json': 48, 'scenarios/boot-entry-lessons-v25.json': 51, 'scenarios/boot-entry-lessons-v26.json': 53, 'scenarios/boot-entry-lessons-v27.json': 55, 'scenarios/boot-entry-lessons-v28.json': 57, 'scenarios/boot-entry-lessons-v29.json': 59, 'scenarios/boot-entry-lessons-v30.json': 61, 'scenarios/boot-entry-lessons-v31.json': 63, 'scenarios/boot-entry-lessons-v32.json': 64, 'scenarios/boot-entry-lessons-v33.json':65, 'scenarios/boot-entry-lessons-v34.json':66, 'scenarios/boot-entry-lessons-v35.json':68}
    target = ('scenarios/boot-entry-lessons-v36.json' if origin.startswith('scenarios/boot-entry-lessons-v') else 'scenarios/kernel-module-curriculum-v2.json' if origin.startswith('scenarios/kernel-module-') else 'scenarios/ldap-administration-v2.json')
    if bundled_path not in {target, 'scenarios/boot-entry-lessons-v37.json'} or origin not in expected:
        return False
    original = json.loads((ROOT / origin).read_text())['scenarios']
    matching = [old for old in original if old['id'] == lab['id']]
    return len(original) == expected[origin] and len(matching) == 1 and matching[0] == lab


def content():
    labs = catalog()
    rows = []
    for shard in SHARDS:
        path = f'scenarios/presentation/{shard}.json'
        for row in json.loads((ROOT / path).read_text()):
            rows.append((row, path))
    if len({row['id'] for row, _ in rows}) != len(rows):
        raise ValueError('Duplicate review ID')
    review = {row['id']: (row, path) for row, path in rows}
    # A reviewed expansion of one existing lesson, not additional lab credit.
    dns_path = ROOT / 'scenarios/presentation/lpic1_direct_dns.json'
    if dns_path.is_symlink() or dns_path.stat().st_size > 16384:
        raise ValueError('Direct DNS teaching exceeds its source-owned bound')
    expanded = json.loads(dns_path.read_text())
    identity = 'lpic1-host-resolver'
    if expanded['id'] != identity or expanded['requirements'] != review[identity][0]['requirements']:
        raise ValueError('Expanded direct DNS teaching changed independent requirements')
    review[identity] = (expanded, 'scenarios/presentation/lpic1_direct_dns.json')
    dns_path = ROOT / 'scenarios/presentation/lpic2_dns_queries.json'
    if dns_path.is_symlink() or dns_path.stat().st_size > 40000:
        raise ValueError('Authoritative query teaching exceeds its source-owned bound')
    additions = json.loads(dns_path.read_text())
    allowed = {'lpic2-dns-dualstack','lpic2-dns-aliases','lpic2-dns-secondary'}
    if len(additions) != 3 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected authoritative query lesson subset')
    for row in additions:
        if row['requirements'] != review[row['id']][0]['requirements']:
            raise ValueError('Query extension changed independent DNS requirements')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_dns_queries.json')
    probe_path = ROOT / 'scenarios/presentation/lpic2_port_probes.json'
    if probe_path.is_symlink() or probe_path.stat().st_size > 20000:
        raise ValueError('Numeric port-probe teaching exceeds its source-owned bound')
    additions = json.loads(probe_path.read_text())
    allowed = {'lpic2-exposure-client-policy', 'lpic2-exposure-listener-retirement'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected numeric port-probe lesson subset')
    for row in additions:
        if row['requirements'] != review[row['id']][0]['requirements']:
            raise ValueError('Port-probe extension changed independent requirements')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_port_probes.json')
    telnet_path = ROOT / 'scenarios/presentation/lpic2_telnet_transport.json'
    if telnet_path.is_symlink() or telnet_path.stat().st_size > 20000:
        raise ValueError('Transport client teaching exceeds its source-owned bound')
    additions = json.loads(telnet_path.read_text())
    allowed = {'lpic2-exposure-client-policy', 'lpic2-exposure-listener-retirement'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected transport client lesson subset')
    for row in additions:
        if row['requirements'] != review[row['id']][0]['requirements']:
            raise ValueError('Transport client extension changed independent requirements')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_telnet_transport.json')
    poe_path = ROOT / 'scenarios/presentation/ccna_poe.json'
    if poe_path.is_symlink() or poe_path.stat().st_size > 20000:
        raise ValueError('Power-plan teaching exceeds its source-owned bound')
    additions = json.loads(poe_path.read_text())
    allowed = {'ccna-poe-compatible-ports', 'ccna-poe-budget-recovery'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected power-plan lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A new power lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/ccna_poe.json')
    kernel_path = ROOT / 'scenarios/presentation/lpic2_kernel_build.json'
    if kernel_path.is_symlink() or kernel_path.stat().st_size > 24000:
        raise ValueError('Kernel-build teaching exceeds its source-owned bound')
    additions = json.loads(kernel_path.read_text())
    allowed = {'lpic2-kernel-root-build', 'lpic2-kernel-staged-handover'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected kernel-build lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A new kernel lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_kernel_build.json')
    dkms_path = ROOT / 'scenarios/presentation/lpic2_dkms.json'
    if dkms_path.is_symlink() or dkms_path.stat().st_size > 20000:
        raise ValueError('DKMS teaching exceeds its source-owned bound')
    additions = json.loads(dkms_path.read_text())
    allowed = {'lpic2-dkms-kernel-recovery', 'lpic2-dkms-version-handover'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected DKMS lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A DKMS lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_dkms.json')
    initramfs_path = ROOT / 'scenarios/presentation/lpic2_initramfs.json'
    if initramfs_path.is_symlink() or initramfs_path.stat().st_size > 20000:
        raise ValueError('early-userspace teaching exceeds its source-owned bound')
    additions = json.loads(initramfs_path.read_text())
    allowed = {'lpic2-initramfs-root-drivers', 'lpic2-initramfs-kernel-handover'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected early-userspace lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A early-userspace lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_initramfs.json')
    pxe_path = ROOT / 'scenarios/presentation/lpic2_pxe.json'
    if pxe_path.is_symlink() or pxe_path.stat().st_size > 20000:
        raise ValueError('network boot teaching exceeds its source-owned bound')
    additions = json.loads(pxe_path.read_text())
    allowed = {'lpic2-pxe-firmware-selection', 'lpic2-pxelinux-client-lookup'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected network boot lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A network boot lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_pxe.json')
    iscsi_path = ROOT / 'scenarios/presentation/lpic2_iscsi.json'
    if iscsi_path.is_symlink() or iscsi_path.stat().st_size > 20000:
        raise ValueError('iSCSI teaching exceeds its source-owned bound')
    additions = json.loads(iscsi_path.read_text())
    allowed = {'lpic2-iscsi-target-identity', 'lpic2-iscsi-startup-policy'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected iSCSI lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A iSCSI lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_iscsi.json')
    ata_smart_path = ROOT / 'scenarios/presentation/lpic2_ata_smart.json'
    if ata_smart_path.is_symlink() or ata_smart_path.stat().st_size > 20000:
        raise ValueError('ATA SMART teaching exceeds its source-owned bound')
    additions = json.loads(ata_smart_path.read_text())
    allowed = {'lpic2-smart-short-diagnosis', 'lpic2-smart-monitor-identity'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected ATA SMART lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A ATA SMART lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_ata_smart.json')
    apache_https_path = ROOT / 'scenarios/presentation/lpic2_apache_https.json'
    if apache_https_path.is_symlink() or apache_https_path.stat().st_size > 20000:
        raise ValueError('Apache HTTPS teaching exceeds its source-owned bound')
    additions = json.loads(apache_https_path.read_text())
    allowed = {'lpic2-apache-https-chain', 'lpic2-apache-https-sni'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected Apache HTTPS lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A Apache HTTPS lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_apache_https.json')
    desktop_profile_path = ROOT / 'scenarios/presentation/lpic1_desktop_profile.json'
    if desktop_profile_path.is_symlink() or desktop_profile_path.stat().st_size > 20000:
        raise ValueError('Desktop profile teaching exceeds its source-owned bound')
    additions = json.loads(desktop_profile_path.read_text())
    allowed = {'lpic1-xorg-keyboard-scope', 'lpic1-remote-desktop-arrangement', 'lpic1-accessible-input-profile'}
    if len(additions) != 3 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected Desktop profile lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A Desktop profile lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/lpic1_desktop_profile.json')
    security_investigation_path = ROOT / 'scenarios/presentation/ccna_security_investigation.json'
    if security_investigation_path.is_symlink() or security_investigation_path.stat().st_size > 20000:
        raise ValueError('Security investigation teaching exceeds its source-owned bound')
    additions = json.loads(security_investigation_path.read_text())
    allowed = {'ccna-management-change-evidence', 'ccna-facility-access-plan'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected Security investigation lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A Security investigation lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/ccna_security_investigation.json')
    credentials_vpn_path = ROOT / 'scenarios/presentation/ccna_credentials_vpn.json'
    if credentials_vpn_path.is_symlink() or credentials_vpn_path.stat().st_size > 20000:
        raise ValueError('Credentials and VPN teaching exceeds its source-owned bound')
    additions = json.loads(credentials_vpn_path.read_text())
    allowed = {'ccna-credential-policy-repair', 'ccna-ipsec-design-review'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected Credentials and VPN lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A Credentials and VPN lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/ccna_credentials_vpn.json')
    staged_network_path = ROOT / 'scenarios/presentation/ccna_staged_network.json'
    if staged_network_path.is_symlink() or staged_network_path.stat().st_size > 20000:
        raise ValueError('Staged network changes teaching exceeds its source-owned bound')
    additions = json.loads(staged_network_path.read_text())
    allowed = {'ccna-controller-pilot-rollout', 'ccna-controller-intent-path'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected Staged network changes lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A Staged network changes lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/ccna_staged_network.json')
    network_evidence_review_path = ROOT / 'scenarios/presentation/ccna_network_evidence_review.json'
    if network_evidence_review_path.is_symlink() or network_evidence_review_path.stat().st_size > 20000:
        raise ValueError('Network evidence review teaching exceeds its source-owned bound')
    additions = json.loads(network_evidence_review_path.read_text())
    allowed = {'ccna-review-ai-operations-claim', 'ccna-review-declarative-change'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected Network evidence review lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A Network evidence review lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/ccna_network_evidence_review.json')
    site_connectivity_path = ROOT / 'scenarios/presentation/ccna_site_connectivity.json'
    if site_connectivity_path.is_symlink() or site_connectivity_path.stat().st_size > 20000:
        raise ValueError('Site connectivity teaching exceeds its source-owned bound')
    additions = json.loads(site_connectivity_path.read_text())
    allowed = {'ccna-diverse-branch-paths', 'ccna-local-and-hosted-connectivity'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected Site connectivity lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A Site connectivity lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/ccna_site_connectivity.json')
    host_dns_security_path = ROOT / 'scenarios/presentation/lpic_host_dns_security.json'
    if host_dns_security_path.is_symlink() or host_dns_security_path.stat().st_size > 20000:
        raise ValueError('Host and DNS security teaching exceeds its source-owned bound')
    additions = json.loads(host_dns_security_path.read_text())
    allowed = {'lpic1-retire-socket-activation', 'lpic2-authenticated-zone-handover'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected Host and DNS security lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A Host and DNS security lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/lpic_host_dns_security.json')
    vlan_gateway_path = ROOT / 'scenarios/presentation/ccna_vlan_gateway.json'
    if vlan_gateway_path.is_symlink() or vlan_gateway_path.stat().st_size > 20000:
        raise ValueError('VLAN gateway repair teaching exceeds its source-owned bound')
    additions = json.loads(vlan_gateway_path.read_text())
    allowed = {'ccna-default-access-membership', 'ccna-switched-gateway-repair'}
    if len(additions) != 2 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected VLAN gateway repair lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A VLAN gateway repair lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/ccna_vlan_gateway.json')
    nfs_export_path = ROOT / 'scenarios/presentation/lpic2_nfs_export_operations.json'
    if nfs_export_path.is_symlink() or nfs_export_path.stat().st_size > 20000:
        raise ValueError('NFS export operations teaching exceeds its source-owned bound')
    additions = json.loads(nfs_export_path.read_text())
    allowed = {'lpic2-withdraw-obsolete-nfs-export'}
    if len(additions) != 1 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected NFS export operations lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A NFS export operations lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/lpic2_nfs_export_operations.json')
    route_reading_path = ROOT / 'scenarios/presentation/ccna_route_reading.json'
    if route_reading_path.is_symlink() or route_reading_path.stat().st_size > 20000:
        raise ValueError('installed route reading teaching exceeds its source-owned bound')
    additions = json.loads(route_reading_path.read_text())
    allowed = {'ccna-route-reading-backup'}
    if len(additions) != 1 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected installed route reading lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A installed route reading lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/ccna_route_reading.json')
    broadcast_path = ROOT / 'scenarios/presentation/ccna_ospf_broadcast.json'
    if broadcast_path.is_symlink() or broadcast_path.stat().st_size > 20000:
        raise ValueError('broadcast designated roles teaching exceeds its source-owned bound')
    additions = json.loads(broadcast_path.read_text())
    allowed = {'ccna-ospf-broadcast-handover'}
    if len(additions) != 1 or {row['id'] for row in additions} != allowed:
        raise ValueError('Unexpected broadcast designated roles lesson subset')
    for row in additions:
        if row['id'] in review:
            raise ValueError('A broadcast designated roles lesson replaced a prior review')
        review[row['id']] = (row, 'scenarios/presentation/ccna_ospf_broadcast.json')
    # Two source-authored prerequisite additions; not extra labs or changed grades.
    note_path = ROOT / 'scenarios/presentation/lpic1_foundations.json'
    if not note_path.is_file() or note_path.is_symlink() or note_path.stat().st_size > 8192:
        raise ValueError('Foundation notes exceed the bundled text limit')
    notes = json.loads(note_path.read_text())
    if set(notes) != {'version','lessons'} or notes['version'] != 1 or len(notes['lessons']) != 2:
        raise ValueError('Unsupported foundation notes')
    seen = set()
    for note in notes['lessons']:
        if (set(note) != {'id','concepts','worked','introduction_review','sources'}
                or note['id'] not in {'lpic1-host-address','lpic1-host-routing'}
                or note['id'] in seen or len(note['concepts']) != 1 or len(note['worked']) != 1
                or len(note['introduction_review']) < 40 or not 1 <= len(note['sources']) <= 3
                or any(not s.startswith('https://www.rfc-editor.org/rfc/') for s in note['sources'])):
            raise ValueError('Invalid lesson-specific foundation notes')
        seen.add(note['id'])
        row, _ = review[note['id']]
        row['review']['introduction'] += (' Added prerequisite notes: ' + note['introduction_review']
            + ' Source: scenarios/presentation/lpic1_foundations.json.')
    if set(review) != {lab['id'] for lab, _ in labs}:
        raise ValueError('Review inventory differs from registered catalog')
    compact = lambda text: ''.join(text.split())
    for lab, path in labs:
        row, _ = review[lab['id']]
        if row['source'] != path and not same_preserved_origin(lab, row['source'], path):
            raise ValueError(f"Incorrect source for {lab['id']}")
        if row['status'] not in ['pending', 'updated', 'already meets', 'blocked']:
            raise ValueError(f"Unknown status for {lab['id']}")
        if compact(' '.join(row['requirements'])) != compact(lab['objective']):
            raise ValueError(f"Changed original requirement characters: {lab['id']}")
        if row['status'] in ['updated', 'already meets']:
            for view in ['introduction', 'example', 'assignment']:
                if len(row['review'][view].strip()) < 40:
                    raise ValueError(f"Missing {view} content review: {lab['id']}")
    return labs, review

def documents():
    labs, review = content()
    counts = Counter(lab['track'] for lab, _ in labs)
    statuses = Counter(row['status'] for row, _ in review.values())
    lines = ['# Existing-lab teaching rollout inventory', '',
             'Derived from the actual review library: preserved normal catalog plus accepted and continuation units.',
             'Statuses record lesson-specific source/content and UI-path review of all three screens,',
             'not merely shared-renderer inheritance. Compiled rendering and representative native',
             'evidence are recorded in TEACHING_ROLLOUT_VERIFICATION.md, GATEWAY_BATCH.md and CONTINUATION.md. This is neither',
             'curriculum-completion nor independent novice-comprehension acceptance.', '',
             f"**{len(labs)} labs:** " + ', '.join(f'{track} {counts[track]}' for track in TRACKS) + '.', '',
             f"**Statuses:** {statuses['updated']} updated; {statuses['already meets']} already meets; "
             f"{statuses['pending']} pending; {statuses['blocked']} blocked.", '',
             '[Shared standard](TEACHING_PRESENTATION.md). Complete per-screen review notes are in',
             'the linked UI-only JSON entries; CSV contains every full note and original source path.', '',
             '| Track | Lab ID | Assignment | Status | Review details |',
             '| --- | --- | --- | --- | --- |']
    out = io.StringIO()
    writer = csv.writer(out, lineterminator='\n')
    writer.writerow(['track','id','title','status','source','introduction_review','example_review','assignment_review'])
    by_id = {lab['id']: lab for lab, _ in labs}
    for track in TRACKS:
        for lab, source in labs:
            if lab['track'] != track:
                continue
            row, shard = review[lab['id']]
            title = lab['title'].replace('|', '\\|')
            lines.append(f"| {track} | `{lab['id']}` | {title} | {row['status']} | [Three-screen notes](../{shard}) |")
            writer.writerow([track,lab['id'],lab['title'],row['status'],source,*[row['review'][view] for view in ['introduction','example','assignment']]])
    lines += ['', '## Exceptions and existing approval', '']
    for row, _ in review.values():
        if row['status'] != 'updated':
            lines.append(f"- `{row['id']}` — **{row['status']}**: {by_id[row['id']]['title']}. {row['review']['introduction']}")
    lines += ['', 'Status above distinguishes completed reviews from pending or blocked work. Registered labs have',
              'Introduction, separate guided practice and independent Assignment paths. The five',
              'already-meets entries retain the accepted CCNA teaching; the shared range-cell alignment',
              'correction applies to their diagrams where relevant. The owner used Show worked answer',
              'during review, so independent novice acceptance remains unverified.', '',
              'Regenerate/check with `python3 tools/dev.py tool teaching_inventory.py --write` / `--check`.', '']
    return {'docs/TEACHING_INVENTORY.md':'\n'.join(lines), 'docs/TEACHING_INVENTORY.csv':out.getvalue()}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    # Fixed explicit switches; no external catalog/script/path selector.
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--write', action='store_true')
    mode.add_argument('--check', action='store_true')
    args = parser.parse_args()
    write = args.write
    for name, value in documents().items():
        path = ROOT / name
        if write:
            path.write_text(value)
        elif not path.is_file() or path.read_text() != value:
            raise ValueError(f'Stale inventory: {name}; regenerate with --write')
    labs, review = content()
    print(f"Teaching inventory: {len(labs)} registered labs; " + ', '.join(f'{k}={v}' for k,v in sorted(Counter(r['status'] for r,_ in review.values()).items())))

if __name__ == '__main__':
    main()
