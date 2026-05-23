import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_zh.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
      : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
    delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
  ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('zh')
  ];

  /// No description provided for @appName.
  ///
  /// In en, this message translates to:
  /// **'ACP UI'**
  String get appName;

  /// No description provided for @appNameDesc.
  ///
  /// In en, this message translates to:
  /// **'ACP UI - Multi-Agent Collaboration Platform'**
  String get appNameDesc;

  /// No description provided for @commonConfirm.
  ///
  /// In en, this message translates to:
  /// **'Confirm'**
  String get commonConfirm;

  /// No description provided for @commonCancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get commonCancel;

  /// No description provided for @commonSave.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get commonSave;

  /// No description provided for @commonDelete.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get commonDelete;

  /// No description provided for @commonEdit.
  ///
  /// In en, this message translates to:
  /// **'Edit'**
  String get commonEdit;

  /// No description provided for @commonClose.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get commonClose;

  /// No description provided for @commonLoading.
  ///
  /// In en, this message translates to:
  /// **'Loading...'**
  String get commonLoading;

  /// No description provided for @commonError.
  ///
  /// In en, this message translates to:
  /// **'Error'**
  String get commonError;

  /// No description provided for @commonSuccess.
  ///
  /// In en, this message translates to:
  /// **'Success'**
  String get commonSuccess;

  /// No description provided for @commonDisconnect.
  ///
  /// In en, this message translates to:
  /// **'Disconnect'**
  String get commonDisconnect;

  /// No description provided for @commonReconnect.
  ///
  /// In en, this message translates to:
  /// **'Reconnect'**
  String get commonReconnect;

  /// No description provided for @commonReconnecting.
  ///
  /// In en, this message translates to:
  /// **'Reconnecting'**
  String get commonReconnecting;

  /// No description provided for @commonConnecting.
  ///
  /// In en, this message translates to:
  /// **'Connecting...'**
  String get commonConnecting;

  /// No description provided for @commonSettings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get commonSettings;

  /// No description provided for @commonWorkingDirectory.
  ///
  /// In en, this message translates to:
  /// **'Working Directory'**
  String get commonWorkingDirectory;

  /// No description provided for @commonSelectFolder.
  ///
  /// In en, this message translates to:
  /// **'Select Folder'**
  String get commonSelectFolder;

  /// No description provided for @commonNewSession.
  ///
  /// In en, this message translates to:
  /// **'New Session'**
  String get commonNewSession;

  /// No description provided for @commonNoAgents.
  ///
  /// In en, this message translates to:
  /// **'No agents available'**
  String get commonNoAgents;

  /// No description provided for @commonNoAgentsConfigured.
  ///
  /// In en, this message translates to:
  /// **'No remote agents configured'**
  String get commonNoAgentsConfigured;

  /// No description provided for @commonOpenSettings.
  ///
  /// In en, this message translates to:
  /// **'Open Settings (⚙) to add'**
  String get commonOpenSettings;

  /// No description provided for @commonPleaseConnect.
  ///
  /// In en, this message translates to:
  /// **'Please connect to an agent to use this feature'**
  String get commonPleaseConnect;

  /// No description provided for @commonWelcomeTitle.
  ///
  /// In en, this message translates to:
  /// **'Welcome to ACP UI'**
  String get commonWelcomeTitle;

  /// No description provided for @commonWelcomeSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Select an agent and create a new session to start'**
  String get commonWelcomeSubtitle;

  /// No description provided for @commonNoSavedSessions.
  ///
  /// In en, this message translates to:
  /// **'No saved sessions'**
  String get commonNoSavedSessions;

  /// No description provided for @commonSavedSessions.
  ///
  /// In en, this message translates to:
  /// **'Saved Sessions'**
  String get commonSavedSessions;

  /// No description provided for @commonSelectAgent.
  ///
  /// In en, this message translates to:
  /// **'Select Agent'**
  String get commonSelectAgent;

  /// No description provided for @commonNoTasks.
  ///
  /// In en, this message translates to:
  /// **'No tasks'**
  String get commonNoTasks;

  /// No description provided for @commonNoEvents.
  ///
  /// In en, this message translates to:
  /// **'No events'**
  String get commonNoEvents;

  /// No description provided for @commonRunningTasks.
  ///
  /// In en, this message translates to:
  /// **'Running Tasks'**
  String get commonRunningTasks;

  /// No description provided for @commonCompletedTasks.
  ///
  /// In en, this message translates to:
  /// **'Completed Tasks'**
  String get commonCompletedTasks;

  /// No description provided for @commonTotalTasks.
  ///
  /// In en, this message translates to:
  /// **'Total Tasks'**
  String get commonTotalTasks;

  /// No description provided for @commonAutoRefresh.
  ///
  /// In en, this message translates to:
  /// **'Auto Refresh'**
  String get commonAutoRefresh;

  /// No description provided for @commonRefresh.
  ///
  /// In en, this message translates to:
  /// **'Refresh'**
  String get commonRefresh;

  /// No description provided for @commonOverallProgress.
  ///
  /// In en, this message translates to:
  /// **'Overall Progress'**
  String get commonOverallProgress;

  /// No description provided for @commonSystemLoad.
  ///
  /// In en, this message translates to:
  /// **'System Load'**
  String get commonSystemLoad;

  /// No description provided for @commonQAStatus.
  ///
  /// In en, this message translates to:
  /// **'QA Status'**
  String get commonQAStatus;

  /// No description provided for @commonReview.
  ///
  /// In en, this message translates to:
  /// **'Review'**
  String get commonReview;

  /// No description provided for @commonTest.
  ///
  /// In en, this message translates to:
  /// **'Test'**
  String get commonTest;

  /// No description provided for @commonPass.
  ///
  /// In en, this message translates to:
  /// **'Pass'**
  String get commonPass;

  /// No description provided for @commonFail.
  ///
  /// In en, this message translates to:
  /// **'Fail'**
  String get commonFail;

  /// No description provided for @commonNA.
  ///
  /// In en, this message translates to:
  /// **'N/A'**
  String get commonNA;

  /// No description provided for @commonAgents.
  ///
  /// In en, this message translates to:
  /// **'Agents'**
  String get commonAgents;

  /// No description provided for @commonTasks.
  ///
  /// In en, this message translates to:
  /// **'Tasks'**
  String get commonTasks;

  /// No description provided for @commonProgress.
  ///
  /// In en, this message translates to:
  /// **'Progress'**
  String get commonProgress;

  /// No description provided for @commonLoad.
  ///
  /// In en, this message translates to:
  /// **'Load'**
  String get commonLoad;

  /// No description provided for @commonStatus.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get commonStatus;

  /// No description provided for @commonIdle.
  ///
  /// In en, this message translates to:
  /// **'Idle'**
  String get commonIdle;

  /// No description provided for @commonBusy.
  ///
  /// In en, this message translates to:
  /// **'Busy'**
  String get commonBusy;

  /// No description provided for @commonOverloaded.
  ///
  /// In en, this message translates to:
  /// **'Overloaded'**
  String get commonOverloaded;

  /// No description provided for @commonPending.
  ///
  /// In en, this message translates to:
  /// **'Pending'**
  String get commonPending;

  /// No description provided for @commonRunning.
  ///
  /// In en, this message translates to:
  /// **'Running'**
  String get commonRunning;

  /// No description provided for @commonCompleted.
  ///
  /// In en, this message translates to:
  /// **'Completed'**
  String get commonCompleted;

  /// No description provided for @commonFailed.
  ///
  /// In en, this message translates to:
  /// **'Failed'**
  String get commonFailed;

  /// No description provided for @commonBlocked.
  ///
  /// In en, this message translates to:
  /// **'Blocked'**
  String get commonBlocked;

  /// No description provided for @commonFlowing.
  ///
  /// In en, this message translates to:
  /// **'Flowing'**
  String get commonFlowing;

  /// No description provided for @navChat.
  ///
  /// In en, this message translates to:
  /// **'Chat'**
  String get navChat;

  /// No description provided for @navMultiAgent.
  ///
  /// In en, this message translates to:
  /// **'Multi-Agent'**
  String get navMultiAgent;

  /// No description provided for @navMultiSession.
  ///
  /// In en, this message translates to:
  /// **'Multi-Session'**
  String get navMultiSession;

  /// No description provided for @navWorkflow.
  ///
  /// In en, this message translates to:
  /// **'Workflow'**
  String get navWorkflow;

  /// No description provided for @navOrchestration.
  ///
  /// In en, this message translates to:
  /// **'Orchestration Monitor'**
  String get navOrchestration;

  /// No description provided for @navAgentTeams.
  ///
  /// In en, this message translates to:
  /// **'Agent Teams'**
  String get navAgentTeams;

  /// No description provided for @navCollaboration.
  ///
  /// In en, this message translates to:
  /// **'Collaboration Network'**
  String get navCollaboration;

  /// No description provided for @navHermes.
  ///
  /// In en, this message translates to:
  /// **'Hermes Dashboard'**
  String get navHermes;

  /// No description provided for @navTaskGraph.
  ///
  /// In en, this message translates to:
  /// **'Task Graph'**
  String get navTaskGraph;

  /// No description provided for @navMemory.
  ///
  /// In en, this message translates to:
  /// **'Memory'**
  String get navMemory;

  /// No description provided for @navErrorMonitor.
  ///
  /// In en, this message translates to:
  /// **'Error Monitor'**
  String get navErrorMonitor;

  /// No description provided for @navEvolution.
  ///
  /// In en, this message translates to:
  /// **'Evolution'**
  String get navEvolution;

  /// No description provided for @navPattern.
  ///
  /// In en, this message translates to:
  /// **'Pattern Library'**
  String get navPattern;

  /// No description provided for @navBotConfig.
  ///
  /// In en, this message translates to:
  /// **'Bot Config'**
  String get navBotConfig;

  /// No description provided for @navGateway.
  ///
  /// In en, this message translates to:
  /// **'Remote Control'**
  String get navGateway;

  /// No description provided for @navStatus.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get navStatus;

  /// No description provided for @navMonitor.
  ///
  /// In en, this message translates to:
  /// **'Monitor'**
  String get navMonitor;

  /// No description provided for @navHistory.
  ///
  /// In en, this message translates to:
  /// **'History'**
  String get navHistory;

  /// No description provided for @hermesTitle.
  ///
  /// In en, this message translates to:
  /// **'Hermes Dashboard'**
  String get hermesTitle;

  /// No description provided for @hermesSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Task Orchestration & Agent Management'**
  String get hermesSubtitle;

  /// No description provided for @hermesNoActiveTasks.
  ///
  /// In en, this message translates to:
  /// **'No active tasks'**
  String get hermesNoActiveTasks;

  /// No description provided for @hermesNoRegisteredAgents.
  ///
  /// In en, this message translates to:
  /// **'No registered agents'**
  String get hermesNoRegisteredAgents;

  /// No description provided for @agentTeamsTitle.
  ///
  /// In en, this message translates to:
  /// **'Agent Teams Platform'**
  String get agentTeamsTitle;

  /// No description provided for @agentTeamsSubtitle.
  ///
  /// In en, this message translates to:
  /// **'3-Platform Sync · Realtime Progress · Human-like Pet · Hermes Monitor'**
  String get agentTeamsSubtitle;

  /// No description provided for @agentTeamsAgents.
  ///
  /// In en, this message translates to:
  /// **'Agents'**
  String get agentTeamsAgents;

  /// No description provided for @agentTeamsThinking.
  ///
  /// In en, this message translates to:
  /// **'Thinking'**
  String get agentTeamsThinking;

  /// No description provided for @agentTeamsExecuting.
  ///
  /// In en, this message translates to:
  /// **'Executing'**
  String get agentTeamsExecuting;

  /// No description provided for @agentTeamsSuccessRate.
  ///
  /// In en, this message translates to:
  /// **'Success Rate'**
  String get agentTeamsSuccessRate;

  /// No description provided for @agentTeamsViewAll.
  ///
  /// In en, this message translates to:
  /// **'All'**
  String get agentTeamsViewAll;

  /// No description provided for @agentTeamsViewProgress.
  ///
  /// In en, this message translates to:
  /// **'Progress'**
  String get agentTeamsViewProgress;

  /// No description provided for @agentTeamsViewCollaboration.
  ///
  /// In en, this message translates to:
  /// **'Collaboration'**
  String get agentTeamsViewCollaboration;

  /// No description provided for @agentTeamsViewPets.
  ///
  /// In en, this message translates to:
  /// **'Pets'**
  String get agentTeamsViewPets;

  /// No description provided for @agentTeamsSyncStatus.
  ///
  /// In en, this message translates to:
  /// **'3-Platform Sync'**
  String get agentTeamsSyncStatus;

  /// No description provided for @agentTeamsSyncConnected.
  ///
  /// In en, this message translates to:
  /// **'Connected'**
  String get agentTeamsSyncConnected;

  /// No description provided for @agentTeamsSyncPending.
  ///
  /// In en, this message translates to:
  /// **'Pending'**
  String get agentTeamsSyncPending;

  /// No description provided for @agentTeamsAgentTeam.
  ///
  /// In en, this message translates to:
  /// **'Agent Team'**
  String get agentTeamsAgentTeam;

  /// No description provided for @agentTeamsWebPlatform.
  ///
  /// In en, this message translates to:
  /// **'Web'**
  String get agentTeamsWebPlatform;

  /// No description provided for @agentTeamsDesktopPlatform.
  ///
  /// In en, this message translates to:
  /// **'Desktop'**
  String get agentTeamsDesktopPlatform;

  /// No description provided for @agentTeamsMobilePlatform.
  ///
  /// In en, this message translates to:
  /// **'Mobile'**
  String get agentTeamsMobilePlatform;

  /// No description provided for @agentTeamsSyncProtocol.
  ///
  /// In en, this message translates to:
  /// **'Sync Protocol'**
  String get agentTeamsSyncProtocol;

  /// No description provided for @agentTeamsLastSync.
  ///
  /// In en, this message translates to:
  /// **'Last Sync'**
  String get agentTeamsLastSync;

  /// No description provided for @agentPetPet.
  ///
  /// In en, this message translates to:
  /// **'Pet'**
  String get agentPetPet;

  /// No description provided for @agentPetPoke.
  ///
  /// In en, this message translates to:
  /// **'Poke'**
  String get agentPetPoke;

  /// No description provided for @agentPetFeed.
  ///
  /// In en, this message translates to:
  /// **'Feed'**
  String get agentPetFeed;

  /// No description provided for @agentPetPlay.
  ///
  /// In en, this message translates to:
  /// **'Play'**
  String get agentPetPlay;

  /// No description provided for @growthSystemLevelNovice.
  ///
  /// In en, this message translates to:
  /// **'Novice'**
  String get growthSystemLevelNovice;

  /// No description provided for @growthSystemLevelApprentice.
  ///
  /// In en, this message translates to:
  /// **'Apprentice'**
  String get growthSystemLevelApprentice;

  /// No description provided for @growthSystemLevelSkilled.
  ///
  /// In en, this message translates to:
  /// **'Skilled'**
  String get growthSystemLevelSkilled;

  /// No description provided for @growthSystemLevelExpert.
  ///
  /// In en, this message translates to:
  /// **'Expert'**
  String get growthSystemLevelExpert;

  /// No description provided for @growthSystemLevelMaster.
  ///
  /// In en, this message translates to:
  /// **'Master'**
  String get growthSystemLevelMaster;

  /// No description provided for @growthSystemLevelUp.
  ///
  /// In en, this message translates to:
  /// **'Level Up!'**
  String get growthSystemLevelUp;

  /// No description provided for @growthSystemExperience.
  ///
  /// In en, this message translates to:
  /// **'Experience'**
  String get growthSystemExperience;

  /// No description provided for @growthSystemCompletedTasks.
  ///
  /// In en, this message translates to:
  /// **'Completed Tasks'**
  String get growthSystemCompletedTasks;

  /// No description provided for @growthSystemSuccessRate.
  ///
  /// In en, this message translates to:
  /// **'Success Rate'**
  String get growthSystemSuccessRate;

  /// No description provided for @growthSystemAchievements.
  ///
  /// In en, this message translates to:
  /// **'Achievements'**
  String get growthSystemAchievements;

  /// No description provided for @growthSystemNextGoal.
  ///
  /// In en, this message translates to:
  /// **'Next Goal:'**
  String get growthSystemNextGoal;

  /// No description provided for @growthSystemUnlockAchievement.
  ///
  /// In en, this message translates to:
  /// **'New Achievement Unlocked!'**
  String get growthSystemUnlockAchievement;

  /// No description provided for @interactionPanelHappiness.
  ///
  /// In en, this message translates to:
  /// **'Happiness'**
  String get interactionPanelHappiness;

  /// No description provided for @interactionPanelAffinity.
  ///
  /// In en, this message translates to:
  /// **'Affinity'**
  String get interactionPanelAffinity;

  /// No description provided for @interactionPanelInteractionActions.
  ///
  /// In en, this message translates to:
  /// **'Interaction Actions'**
  String get interactionPanelInteractionActions;

  /// No description provided for @interactionPanelPet.
  ///
  /// In en, this message translates to:
  /// **'Pet'**
  String get interactionPanelPet;

  /// No description provided for @interactionPanelPoke.
  ///
  /// In en, this message translates to:
  /// **'Poke'**
  String get interactionPanelPoke;

  /// No description provided for @interactionPanelFeed.
  ///
  /// In en, this message translates to:
  /// **'Feed'**
  String get interactionPanelFeed;

  /// No description provided for @interactionPanelPlay.
  ///
  /// In en, this message translates to:
  /// **'Play'**
  String get interactionPanelPlay;

  /// No description provided for @interactionPanelPetEffect.
  ///
  /// In en, this message translates to:
  /// **'+5 Happiness'**
  String get interactionPanelPetEffect;

  /// No description provided for @interactionPanelPokeEffect.
  ///
  /// In en, this message translates to:
  /// **'-2 Happiness'**
  String get interactionPanelPokeEffect;

  /// No description provided for @interactionPanelFeedEffect.
  ///
  /// In en, this message translates to:
  /// **'+10 Happiness +5 Affinity'**
  String get interactionPanelFeedEffect;

  /// No description provided for @interactionPanelPlayEffect.
  ///
  /// In en, this message translates to:
  /// **'+15 Happiness +10 Affinity'**
  String get interactionPanelPlayEffect;

  /// No description provided for @interactionPanelInteractionStats.
  ///
  /// In en, this message translates to:
  /// **'Interaction Stats'**
  String get interactionPanelInteractionStats;

  /// No description provided for @interactionPanelPetCount.
  ///
  /// In en, this message translates to:
  /// **'Pet Count'**
  String get interactionPanelPetCount;

  /// No description provided for @interactionPanelLastInteraction.
  ///
  /// In en, this message translates to:
  /// **'Last Interaction'**
  String get interactionPanelLastInteraction;

  /// No description provided for @interactionPanelNone.
  ///
  /// In en, this message translates to:
  /// **'None'**
  String get interactionPanelNone;

  /// No description provided for @interactionPanelTip.
  ///
  /// In en, this message translates to:
  /// **'Regular interactions boost Agent happiness and affinity, unlock more expressions and animations!'**
  String get interactionPanelTip;

  /// No description provided for @agentProgressIdleState.
  ///
  /// In en, this message translates to:
  /// **'Agent idle, waiting for new tasks...'**
  String get agentProgressIdleState;

  /// No description provided for @agentProgressErrorState.
  ///
  /// In en, this message translates to:
  /// **'Execution error, please check logs'**
  String get agentProgressErrorState;

  /// No description provided for @agentProgressTasksCompleted.
  ///
  /// In en, this message translates to:
  /// **'Completed'**
  String get agentProgressTasksCompleted;

  /// No description provided for @agentProgressTasksFailed.
  ///
  /// In en, this message translates to:
  /// **'Failed'**
  String get agentProgressTasksFailed;

  /// No description provided for @agentProgressToolCalls.
  ///
  /// In en, this message translates to:
  /// **'Tool Calls'**
  String get agentProgressToolCalls;

  /// No description provided for @agentProgressThinkingTime.
  ///
  /// In en, this message translates to:
  /// **'Thinking Time'**
  String get agentProgressThinkingTime;

  /// No description provided for @agentProgressSuccessRate.
  ///
  /// In en, this message translates to:
  /// **'Success Rate'**
  String get agentProgressSuccessRate;

  /// No description provided for @agentProgressThinkingProcess.
  ///
  /// In en, this message translates to:
  /// **'Thinking Process'**
  String get agentProgressThinkingProcess;

  /// No description provided for @agentProgressThinkingRealtime.
  ///
  /// In en, this message translates to:
  /// **'Thinking in realtime...'**
  String get agentProgressThinkingRealtime;

  /// No description provided for @agentProgressThinkingChunksTimeline.
  ///
  /// In en, this message translates to:
  /// **'Thinking Chunks Timeline'**
  String get agentProgressThinkingChunksTimeline;

  /// No description provided for @agentProgressParameters.
  ///
  /// In en, this message translates to:
  /// **'Parameters'**
  String get agentProgressParameters;

  /// No description provided for @agentProgressExecutionResult.
  ///
  /// In en, this message translates to:
  /// **'Execution Result'**
  String get agentProgressExecutionResult;

  /// No description provided for @agentProgressErrorMessage.
  ///
  /// In en, this message translates to:
  /// **'Error Message'**
  String get agentProgressErrorMessage;

  /// No description provided for @agentProgressCancelExecution.
  ///
  /// In en, this message translates to:
  /// **'Cancel Execution'**
  String get agentProgressCancelExecution;

  /// No description provided for @agentProgressAgentOutput.
  ///
  /// In en, this message translates to:
  /// **'Agent Output'**
  String get agentProgressAgentOutput;

  /// No description provided for @agentProgressOutputRealtime.
  ///
  /// In en, this message translates to:
  /// **'Outputting'**
  String get agentProgressOutputRealtime;

  /// No description provided for @agentProgressOutputChunks.
  ///
  /// In en, this message translates to:
  /// **'Output Chunks'**
  String get agentProgressOutputChunks;

  /// No description provided for @agentProgressWaitingPermission.
  ///
  /// In en, this message translates to:
  /// **'Waiting for Permission'**
  String get agentProgressWaitingPermission;

  /// No description provided for @agentProgressTimeoutWarning.
  ///
  /// In en, this message translates to:
  /// **'Permission will timeout in {seconds} seconds'**
  String agentProgressTimeoutWarning(int seconds);

  /// No description provided for @agentProgressSelectAction.
  ///
  /// In en, this message translates to:
  /// **'Select action:'**
  String get agentProgressSelectAction;

  /// No description provided for @agentProgressRecommended.
  ///
  /// In en, this message translates to:
  /// **'Recommended'**
  String get agentProgressRecommended;

  /// No description provided for @agentProgressResponding.
  ///
  /// In en, this message translates to:
  /// **'Responding...'**
  String get agentProgressResponding;

  /// No description provided for @emotionDisplayHappy.
  ///
  /// In en, this message translates to:
  /// **'Feeling happy'**
  String get emotionDisplayHappy;

  /// No description provided for @emotionDisplayFocused.
  ///
  /// In en, this message translates to:
  /// **'Focused on work'**
  String get emotionDisplayFocused;

  /// No description provided for @emotionDisplayConfused.
  ///
  /// In en, this message translates to:
  /// **'Feeling confused'**
  String get emotionDisplayConfused;

  /// No description provided for @emotionDisplayTired.
  ///
  /// In en, this message translates to:
  /// **'Feeling tired'**
  String get emotionDisplayTired;

  /// No description provided for @emotionDisplayBored.
  ///
  /// In en, this message translates to:
  /// **'Bored waiting'**
  String get emotionDisplayBored;

  /// No description provided for @emotionDisplayExcited.
  ///
  /// In en, this message translates to:
  /// **'Excited'**
  String get emotionDisplayExcited;

  /// No description provided for @emotionDisplayIntensity.
  ///
  /// In en, this message translates to:
  /// **'Emotion Intensity'**
  String get emotionDisplayIntensity;

  /// No description provided for @emotionDisplayRecentEmotion.
  ///
  /// In en, this message translates to:
  /// **'Recent Emotion'**
  String get emotionDisplayRecentEmotion;

  /// No description provided for @languageSelectLanguage.
  ///
  /// In en, this message translates to:
  /// **'Select Language'**
  String get languageSelectLanguage;

  /// No description provided for @languageZhCN.
  ///
  /// In en, this message translates to:
  /// **'Chinese'**
  String get languageZhCN;

  /// No description provided for @languageEnUS.
  ///
  /// In en, this message translates to:
  /// **'English'**
  String get languageEnUS;

  /// No description provided for @languageDeDE.
  ///
  /// In en, this message translates to:
  /// **'German'**
  String get languageDeDE;

  /// No description provided for @languageEsES.
  ///
  /// In en, this message translates to:
  /// **'Spanish'**
  String get languageEsES;

  /// No description provided for @languageRuRU.
  ///
  /// In en, this message translates to:
  /// **'Russian'**
  String get languageRuRU;

  /// No description provided for @languageJaJP.
  ///
  /// In en, this message translates to:
  /// **'Japanese'**
  String get languageJaJP;

  /// No description provided for @languageKoKR.
  ///
  /// In en, this message translates to:
  /// **'Korean'**
  String get languageKoKR;

  /// No description provided for @languageViVN.
  ///
  /// In en, this message translates to:
  /// **'Vietnamese'**
  String get languageViVN;

  /// No description provided for @languageThTH.
  ///
  /// In en, this message translates to:
  /// **'Thai'**
  String get languageThTH;

  /// No description provided for @languageMsMY.
  ///
  /// In en, this message translates to:
  /// **'Malay'**
  String get languageMsMY;

  /// No description provided for @languageFrFR.
  ///
  /// In en, this message translates to:
  /// **'French'**
  String get languageFrFR;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'zh'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'zh':
      return AppLocalizationsZh();
  }

  throw FlutterError(
      'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
      'an issue with the localizations generation tool. Please file an issue '
      'on GitHub with a reproducible sample app and the gen-l10n configuration '
      'that was used.');
}
