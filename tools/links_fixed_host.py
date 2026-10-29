"""UNWIRED source prototype: fixed Linux fixture observations, not a learner API.

Do not execute until a separately reviewed collector/worker verifies exact Root
approval, source/ELF/kernel/Python pins, shared runner lock, fresh owned fixture
identity and a bounded child deadline. No standalone or arbitrary path CLI.
This adapter itself does NOT implement that acceptance boundary.
"""
import errno
import os
import stat

IDS = (
    'shared-alias-independent-copy', 'unlink-one-then-last-name',
    'move-relative-link-changes-parent', 'dangling-link-arrival-retirement',
    'internal-directory-link-prefix', 'copy-follows-source-with-independent-data',
    'fixed-file-errors-preserve-content', 'utf8-data-and-literal-names',
)
NAMES = (
    frozenset(('item', 'alias', 'copy')),
    frozenset(('item', 'alias')),
    frozenset(('a', 'b', 'a/data', 'b/data', 'a/ref', 'b/ref')),
    frozenset(('later', 'latest')),
    frozenset(('real', 'real/data', 'shortcut', 'shortcut/data', 'shortcut/')),
    frozenset(('original', 'view', 'copy')),
    frozenset(('item', 'existing', 'item/child', 'item/', 'absent')),
    frozenset(('-odd name', 'alias name', 'view name')),
)
SYMBOLIC = ({}, {}, {'a/ref':'data', 'b/ref':'data'}, {'latest':'later'},
            {'shortcut':'real'}, {'view':'original'}, {}, {'view name':'-odd name'})

class FixedFixture:
    """Only a trusted, reviewed worker may supply this newly owned directory FD."""
    def __init__(self, index, fd):
        if type(index) is not int or not 0 <= index < len(IDS):
            raise ValueError('Not one of the eight fixed cases')
        info = os.fstat(fd)
        if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
            raise ValueError('Fixture must be private and owned')
        if os.listdir(fd):
            raise ValueError('Fixture must be newly empty; no reuse or cleanup')
        self.index, self.fd, self.identity, self.operations = index, fd, (info.st_dev, info.st_ino), 0

    def path(self, name):
        self.operations += 1
        if self.operations > 128:
            raise ValueError('Fixed operation bound')
        if name not in NAMES[self.index] or name.startswith('/') or '\\' in name:
            raise ValueError('Only exact authored relative paths are admitted')
        if any(p in ('.', '..') for p in name.split('/')):
            raise ValueError('No parent/current-directory traversal in host rows')
        info = os.fstat(self.fd)
        if not stat.S_ISDIR(info.st_mode) or (info.st_dev, info.st_ino) != self.identity:
            raise ValueError('Fixture directory identity changed')
        return name

    def parent(self, name):
        # Mutation parents never follow symbolic prefixes. Only read/inspection
        # deliberately follows the fixed directory-link comparison graph.
        name=self.path(name);parts=name.split('/')
        if not parts[-1]:raise ValueError('Mutation requires a final basename')
        fd=os.dup(self.fd)
        try:
            for component in parts[:-1]:
                child=os.open(component,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=fd)
                os.close(fd);fd=child
            result=fd;fd=-1;return result,parts[-1]
        finally:
            if fd>=0:os.close(fd)

    def make_dir(self, name):
        parent,leaf=self.parent(name)
        try:os.mkdir(leaf,0o700,dir_fd=parent)
        finally:os.close(parent)

    def inspect(self, name, follow=False):
        return os.stat(self.path(name), dir_fd=self.fd, follow_symlinks=follow)

    def inode(self, name, follow=False):
        s = self.inspect(name, follow)
        return s.st_dev, s.st_ino

    def read(self, name):
        # Following is deliberate only inside the fixed, owned, reviewed graph.
        # The worker must reject every unexpected entry/target before invoking us.
        fd = os.open(self.path(name), os.O_RDONLY | os.O_CLOEXEC, dir_fd=self.fd)
        try:
            if not stat.S_ISREG(os.fstat(fd).st_mode):
                raise ValueError('Fixed read must resolve to a regular file')
            data = os.read(fd, 8193)
            if len(data) > 8192:
                raise ValueError('Fixed file exceeded8KiB')
            return data.decode('utf-8')
        finally:
            os.close(fd)

    def write(self, name, text, create=False):
        data = text.encode('utf-8')
        if len(data) > 8192:
            raise ValueError('Fixed content exceeds8KiB')
        parent,leaf=self.parent(name)
        try:self._write(parent,leaf,data,create)
        finally:os.close(parent)

    def _write(self, parent, name, data, create):
        flags = os.O_WRONLY | os.O_CLOEXEC | os.O_NOFOLLOW
        previous = None
        if create:
            flags |= os.O_CREAT | os.O_EXCL
        else:
            previous = os.stat(name, dir_fd=parent, follow_symlinks=False)
            if not stat.S_ISREG(previous.st_mode):
                raise ValueError('Existing writes must target a known regular name')
        fd = os.open(name, flags, 0o600, dir_fd=parent)
        try:
            info = os.fstat(fd)
            if not stat.S_ISREG(info.st_mode) or (previous and
                (previous.st_dev, previous.st_ino) != (info.st_dev, info.st_ino)):
                raise ValueError('Fixed write identity changed')
            os.ftruncate(fd, 0)
            at = 0
            for _ in range(8):
                if at == len(data):
                    return
                written = os.write(fd, data[at:])
                if written <= 0:
                    raise ValueError('Fixed write made no progress')
                at += written
            if at != len(data):
                raise ValueError('Fixed write exceeded finite chunk bound')
        finally:
            os.close(fd)

    def link(self, source, destination):
        src,leaf=self.parent(source)
        try:
            dst,name=self.parent(destination)
            try:os.link(leaf,name,src_dir_fd=src,dst_dir_fd=dst,follow_symlinks=False)
            finally:os.close(dst)
        finally:os.close(src)

    def symbolic(self, target, destination):
        if SYMBOLIC[self.index].get(destination) != target:
            raise ValueError('Only exact authored internal symbolic targets')
        if target.startswith('/') or any(p in ('.', '..') for p in target.split('/')):
            raise ValueError('No absolute or traversal targets in host comparisons')
        parent,leaf=self.parent(destination)
        try:os.symlink(target,leaf,dir_fd=parent)
        finally:os.close(parent)

    def target(self, name):
        return os.readlink(self.path(name), dir_fd=self.fd)

    def unlink(self, name):
        # Only this new fixture's explicitly authored regular/symbolic names.
        parent,leaf=self.parent(name)
        try:os.unlink(leaf,dir_fd=parent)
        finally:os.close(parent)

    def move_unused(self, source, destination):
        # This reference only compares moves to a known unused destination.
        # It does NOT claim rename() generally has the VM's no-replace policy.
        try:
            self.inspect(destination)
        except FileNotFoundError:
            pass
        else:
            raise ValueError('Fixed rename destination must be unused')
        src,leaf=self.parent(source)
        try:
            dst,name=self.parent(destination)
            try:os.rename(leaf,name,src_dir_fd=src,dst_dir_fd=dst)
            finally:os.close(dst)
        finally:os.close(src)

    def error(self, operation):
        try:
            operation()
        except OSError as e:
            if e.errno not in (errno.ENOENT, errno.ENOTDIR, errno.EEXIST):
                raise
            return {errno.ENOENT:'NotFound', errno.ENOTDIR:'NotDirectory', errno.EEXIST:'Exists'}[e.errno]
        raise ValueError('Expected a fixed real syscall error')


def observe_fixed_in_owned_fd(index, fd):
    """No actions, content, path or source may come from JSON or learner data."""
    h = FixedFixture(index, fd)
    if index == 0:
        h.write('item', 'seed\n', True); h.link('item', 'alias'); h.write('copy', h.read('item'), True)
        identity = h.inode('item') == h.inode('alias'); separate = h.inode('item') != h.inode('copy')
        count, copy_count = h.inspect('item').st_nlink, h.inspect('copy').st_nlink
        h.write('alias', 'updated\n')
        fields = dict(same_alias=identity, copy_distinct=separate, alias_links=count, copy_links=copy_count,
                      original=h.read('item'), alias=h.read('alias'), copy=h.read('copy'))
    elif index == 1:
        h.write('item', 'payload\n', True); h.link('item', 'alias'); before = h.inode('item')
        h.unlink('item'); alias = h.inspect('alias'); content = h.read('alias')
        removed = h.error(lambda: h.read('item')); h.unlink('alias')
        fields = dict(alias_identity_preserved=before == (alias.st_dev, alias.st_ino), remaining_links=alias.st_nlink,
                      content=content, removed_name=removed, last_name=h.error(lambda: h.read('alias')))
    elif index == 2:
        h.make_dir('a'); h.make_dir('b'); h.write('a/data', 'alpha\n', True); h.write('b/data', 'beta\n', True)
        h.symbolic('data', 'a/ref'); before = h.inode('a/ref'); content = h.read('a/ref')
        h.move_unused('a/ref', 'b/ref'); after = h.inspect('b/ref')
        fields = dict(before=content, after=h.read('b/ref'), target=h.target('b/ref'),
                      same_link_identity=before == (after.st_dev, after.st_ino), kind_symbolic=stat.S_ISLNK(after.st_mode),
                      old_name=h.error(lambda: h.inspect('a/ref')))
    elif index == 3:
        h.symbolic('later', 'latest'); link = h.inspect('latest'); missing = h.error(lambda: h.read('latest'))
        h.write('later', 'arrived\n', True); arrived = h.read('latest'); h.unlink('later')
        fields = dict(kind_symbolic=stat.S_ISLNK(link.st_mode), target=h.target('latest'), target_bytes=link.st_size,
                      before=missing, arrived=arrived, after=h.error(lambda: h.read('latest')))
    elif index == 4:
        h.make_dir('real'); h.write('real/data', 'nested\n', True); h.symbolic('real', 'shortcut')
        fields = dict(same_target=h.inode('shortcut/data', True) == h.inode('real/data'), content=h.read('shortcut/data'),
                      raw_target=h.target('shortcut'), default_symbolic=stat.S_ISLNK(h.inspect('shortcut').st_mode),
                      follow_directory=stat.S_ISDIR(h.inspect('shortcut', True).st_mode),
                      trailing_slash_directory=stat.S_ISDIR(h.inspect('shortcut/').st_mode))
    elif index == 5:
        h.write('original', 'first\n', True); h.symbolic('original', 'view'); h.write('copy', h.read('view'), True)
        separate = h.inode('original') != h.inode('copy'); h.write('original', 'second\n')
        fields = dict(copy_distinct=separate, copy_regular=stat.S_ISREG(h.inspect('copy').st_mode),
                      source_symbolic=stat.S_ISLNK(h.inspect('view').st_mode), view=h.read('view'), copy=h.read('copy'),
                      copy_links=h.inspect('copy').st_nlink)
    elif index == 6:
        h.write('item', 'protected\n', True); h.write('existing', 'existing\n', True)
        before = (h.inode('item'), h.inode('existing'), h.inspect('item').st_nlink, h.inspect('existing').st_nlink)
        exists = h.error(lambda: h.link('item', 'existing'))
        after = (h.inode('item'), h.inode('existing'), h.inspect('item').st_nlink, h.inspect('existing').st_nlink)
        fields = dict(existing_destination=exists, file_component=h.error(lambda: h.read('item/child')),
                      file_trailing_slash=h.error(lambda: h.inspect('item/', True)), missing=h.error(lambda: h.read('absent')),
                      source=h.read('item'), existing=h.read('existing'), atomic=before == after)
    else:  # Constructor has already closed index to the exact eight cases.
        h.write('-odd name', 'é\n', True); h.link('-odd name', 'alias name'); h.symbolic('-odd name', 'view name')
        fields = dict(same_alias=h.inode('-odd name') == h.inode('alias name'), data_bytes=h.inspect('-odd name').st_size,
                      target_bytes=h.inspect('view name').st_size, target=h.target('view name'), content=h.read('view name'),
                      links=h.inspect('alias name').st_nlink)
    return {'id': IDS[index], 'observations': fields}

if __name__ == '__main__':
    raise SystemExit('UNWIRED SOURCE PROTOTYPE: no host execution until a reviewed approved collector/worker exists')
