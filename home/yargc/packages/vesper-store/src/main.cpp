#include <QFileInfo>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QProcess>
#include <QUrl>

namespace {
QString runCore(const QStringList &arguments) {
    const QString core = qEnvironmentVariable("VESPER_STORE_CORE");
    if (core.isEmpty() || !QFileInfo::exists(core))
        return QStringLiteral("{}");

    QProcess process;
    process.start(core, arguments);
    if (!process.waitForFinished(3000) || process.exitStatus() != QProcess::NormalExit || process.exitCode() != 0)
        return QStringLiteral("{}");

    return QString::fromUtf8(process.readAllStandardOutput()).trimmed();
}
}

int main(int argc, char *argv[]) {
    QGuiApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("Vesper Store"));
    app.setApplicationDisplayName(QStringLiteral("Vesper Store"));
    app.setOrganizationName(QStringLiteral("Vesper"));
    app.setDesktopFileName(QStringLiteral("io.vesper.Store"));

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(
        QStringLiteral("StoreCatalogStatusJson"),
        runCore({QStringLiteral("catalog-status")})
    );
    engine.rootContext()->setContextProperty(
        QStringLiteral("StoreSourcesJson"),
        runCore({QStringLiteral("sources")})
    );

    const QString qmlPath = qEnvironmentVariable("VESPER_STORE_QML");
    if (qmlPath.isEmpty() || !QFileInfo::exists(qmlPath))
        return 2;

    engine.load(QUrl::fromLocalFile(qmlPath));
    if (engine.rootObjects().isEmpty())
        return 3;

    return app.exec();
}
