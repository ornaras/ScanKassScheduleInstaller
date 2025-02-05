# Библиотека для установки SkatWorkerAPI

## Методы присутствующие в библиотеке

- Install - асинхронный метод, скачивающий и устанавливающий SkatWorkerAPI и его зависимости и возвращает код (0 - успешно установлено)
- IsInstalled - функция возвращает `true` если SkatWorkerAPI найден в системе, иначе `false`

## Подводные камни

- Поддерживается только Windows на архитектурах AMD64 и x86
- Установка всех зависимостей SkatWorkerAPI производится не в "тихом" режиме
- Если репозиторий (StarkovVV18/SkatWorker)[https://github.com/StarkovVV18/SkatWorker] станет закрытым, то от библиотеки не будет толку.
- `IsInstalled` определяет присутствует ли файл `C:\ScanKassWorker\SkatWorkerAPI.exe`
- Библиотека собрана только для x86-приложений

## Пример внедрения в .NET Framework проект

```csharp
using System.Runtime.InteropServices;

public static class Program
{
    static void Main()
    {
        if (SkatWorker.IsInstalled())
            Console.WriteLine("installed");
        else
            SkatWorker.Install();
        Console.WriteLine("finish");
    }  
}

internal static class SkatWorker
{
    public const string DllName = "skatworker_installer.dll";

    [DllImport(DllName, EntryPoint = "install")]
    public static extern int Install();
    
    [DllImport(DllName, EntryPoint = "install")]
    public static extern bool IsInstalled();
}
```